use std::collections::{HashSet, VecDeque};

use askama::Template;
use bytes::{BufMut, Bytes, BytesMut};
use ordered_float::OrderedFloat;
use serde_json::Value;
use sqlx::PgPool;
use tron_app::{
    send_sse_msg_to_client,
    tron_components::{
        d3_plot::SseD3PlotTriggerMsg,
        div::{clean_div_with_context, update_and_send_div_with_context},
        tn_future, TnAsset, TnComponentBase, TnContext, TnEvent, TnFutureHTMLResponse,
    },
    tron_macro::ComponentBase,
    TnServerEventData, TRON_APP,
};

use std::collections::HashMap;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;

use crate::{
    embedding_service::{get_all_points, vector_query_and_sort_points, ChunkPoint},
    DB_POOL,
};

use tokio::runtime::Runtime;

static D3PLOT: &str = "d3_plot";
static TOP_HIT_DIV: &str = "top_hit_textarea";
static RESET_BUTTON: &str = "reset_button";

static CMAP: [&str; 97] = [
    "#870098", "#00aaa5", "#3bff00", "#ec0000", "#00a2c3", "#00f400", "#ff1500", "#0092dd",
    "#f9d700", "#0000c9", "#009b13", "#efed00", "#0300aa", "#00a773", "#ccf900", "#63009e",
    "#00aa98", "#84ff00", "#e10000", "#00a7b3", "#00ff00", "#f90000", "#009bd7", "#00ea00",
    "#ff4500", "#0088dd", "#00d200", "#ffa100", "#005ddd", "#00bc00", "#ffc100", "#0013dd",
    "#00a400", "#f7dd00", "#0000c1", "#009f33", "#e8f000", "#1800a7", "#00aa88", "#c4fc00",
    "#00dc00", "#ff8100", "#007ddd", "#00c700", "#ffb100", "#0038dd", "#00af00", "#fcd200",
    "#0000d5", "#009a00", "#f1e700", "#0000b1", "#00a55d", "#d4f700", "#4300a2", "#00aa93",
    "#a1ff00", "#dc0000", "#00aaab", "#1dff00", "#f40000", "#009fcb", "#00ef00", "#ff2d00",
    "#008ddd", "#00d700", "#ff9900", "#0078dd", "#00c200", "#ffb900", "#0025dd", "#00aa00",
    "#78009b", "#00aaa0", "#67ff00", "#e60000", "#00a4bb", "#00fa00", "#fe0000", "#0098dd",
    "#00e200", "#ff5d00", "#0082dd", "#00cc00", "#ffa900", "#004bdd", "#00b400", "#ffc900",
    "#0000dd", "#009f00", "#f4e200", "#0000b9", "#00a248", "#dcf400", "#2d00a4", "#00aa8d",
    "#bcff00",
];

#[non_exhaustive]
#[derive(ComponentBase)]
pub struct AssetSpacePlot<'a: 'static> {
    base: TnComponentBase<'a>,
    html: String,
    db_pool: Option<PgPool>,
}

impl<'a: 'static> AssetSpacePlot<'a> {
    pub async fn init_db_pool(&mut self) {
        self.db_pool = Some(DB_POOL.clone());
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "asset_space.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
pub struct ShowSingleAssetTemplate {
    asset_name: String,
    d3_plot: String,
    top_hit_div: String,
    reset_button: String,
    html_table: String,
}

impl<'a: 'static> AssetSpacePlotBuilder<'a> {
    pub fn init(mut self, tnid: String, title: String, context: &mut TnContextBase) -> Self {
        let component_type = TnComponentType::UserDefined("div".into());

        self.base = TnComponentBase::builder(self.base)
            .init("div".into(), tnid, component_type)
            .set_value(TnComponentValue::String(title))
            .build();

        let d3_plot_script = include_str!("../templates/d3_plot_script.html").to_string();
        let d3_plot = TnD3Plot::builder()
            .init(D3PLOT.into(), d3_plot_script)
            .set_attr(
                "hx-vals",
                r##"js:{event_data:get_event_with_transformed_coordinate(event)}"##,
            )
            .set_action(TnActionExecutionMethod::Await, d3_plot_clicked)
            .build();
        let top_hit_div = TnDiv::builder()
            .init(TOP_HIT_DIV.into(), "".into())
            .set_attr("class", "flex flex-col w-full h-full")
            .set_attr("style", "resize:none; overflow-y: auto;")
            .build();
        let reset_button = TnButton::builder()
            .init(RESET_BUTTON.into(), "Reset".into())
            .set_attr(
                "class",
                "btn btn-sm btn-outline btn-primary w-full h-min p-1",
            )
            .set_attr("hx-target", &format!("#{D3PLOT}"))
            .set_attr("hx-swap", "none")
            .set_action(TnActionExecutionMethod::Await, reset_button_clicked)
            .build();

        context.add_component(d3_plot);
        context.add_component(top_hit_div);
        context.add_component(reset_button);

        {
            // fill in the plot stream data
            let mut stream_data_guard = context.stream_data.blocking_write();
            stream_data_guard.insert(
                "plot_data".into(),
                ("application/text".into(), VecDeque::default()),
            );
            let mut data = VecDeque::default();
            let two_d_embeddding = "x,y,c,o\n".to_string();
            let two_d_embeddding = BytesMut::from_iter(two_d_embeddding.as_bytes());
            data.push_back(two_d_embeddding);
            stream_data_guard.insert("plot_data".into(), ("application/text".into(), data));
        }

        self
    }
}

#[async_trait]
impl<'a> TnComponentRenderTrait<'a> for AssetSpacePlot<'a>
where
    'a: 'static,
{
    /// Generates the internal HTML representation of the button component.
    async fn render(&self) -> String {
        self.html.clone()
    }

    /// Generates the initial HTML representation of the button component.
    async fn initial_render(&self) -> String {
        self.render().await
    }

    async fn pre_render(&mut self, ctx: &TnContextBase) {
        let comp_guard = ctx.components.read().await;

        let d3_plot_output_html = comp_guard
            .get(D3PLOT)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let top_hit_div_output_html = comp_guard
            .get(TOP_HIT_DIV)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        let reset_button_output_html = comp_guard
            .get(RESET_BUTTON)
            .unwrap()
            .read()
            .await
            .initial_render()
            .await;

        {
            let mut two_d_embeddding = "x,y,c,o\n".to_string();
            let asset_id = {
                let assets_guard = ctx.assets.read().await;
                if let Some(TnAsset::U32(chat_id)) = assets_guard.get("asset_id") {
                    *chat_id as i32
                } else {
                    panic!("no chat id found")
                }
            };
            let all_chunks = get_all_points(asset_id)
                .await
                .iter()
                .map(|p| p.chunk.clone())
                .collect::<Vec<_>>();
            two_d_embeddding.extend([all_chunks
                .iter()
                .map(|c| {
                    let fid = c.get_fid();
                    let two_d_embedding = c.two_d_embedding.unwrap();
                    format!(
                        "{},{},{},0.8",
                        two_d_embedding.0,
                        two_d_embedding.1,
                        CMAP[(fid % 97) as usize]
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")]);
            let two_d_embeddding = BytesMut::from_iter(two_d_embeddding.as_bytes());
            {
                let context_guard = ctx;
                let mut stream_data_guard = context_guard.stream_data.write().await;
                let data = stream_data_guard.get_mut("plot_data").unwrap();
                data.1.clear();
                // tracing::info!(target: "tron_app", "length:{}", two_d_embeddding.len());
                data.1.push_back(two_d_embeddding);
                // tracing::info!(target: "tron_app", "stream_data {:?}", data.1[0].len());
            }
        }
        if self.db_pool.as_mut().is_none() {
            self.init_db_pool().await;
        }

        let asset_id = {
            let assets_guard = ctx.assets.read().await;
            if let Some(TnAsset::U32(chat_id)) = assets_guard.get("asset_id") {
                *chat_id as i32
            } else {
                panic!("no asset id found")
            }
        };

        let asset_name = {
            let query = sqlx::query!(
                r#"SELECT a.name
                FROM assets a 
                WHERE asset_id = $1
                "#,
                asset_id
            )
            .fetch_one(self.db_pool.as_ref().unwrap())
            .await
            .expect("db error: getting asset's name");
            query.name
        };

        let html_table = {
            let query = sqlx::query!(
                r#"SELECT t.filename, t.title, COUNT(*) as count
                FROM text_embedding t 
                JOIN assets a ON t.asset_id = a.asset_id
                WHERE a.asset_id = $1
                GROUP BY t.filename, t.title
                "#,
                asset_id
            )
            .fetch_all(self.db_pool.as_ref().unwrap())
            .await
            .expect("db error: getting assets");

            let mut html_table = String::from(r#"<table class="table p-4"><tr><th>Filename</th><th>Title</th><th>Count</th></tr>"#);
            for row in query {
                html_table.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    row.filename.unwrap(), row.title.unwrap(), row.count.unwrap()
                ));
            }
            html_table.push_str("</table>");

            html_table

        };
        self.html = ShowSingleAssetTemplate {
            d3_plot: d3_plot_output_html,
            top_hit_div: top_hit_div_output_html,
            reset_button: reset_button_output_html,
            html_table,
            asset_name
        }
        .render()
        .unwrap()
    }

    async fn post_render(&mut self, _ctx: &TnContextBase) {}
}

fn get_plot_data(all_points_sorted: &[ChunkPoint]) -> String {
    let mut color_scale = 1.0;
    let mut d_color = 8.0 * color_scale / (all_points_sorted.len() as f64);

    let mut two_d_embeddding = "x,y,c,o\n".to_string();
    two_d_embeddding.extend(
        all_points_sorted
            .iter()
            .map(|p| {
                let c = p.chunk.clone();

                let fid = c.get_fid();

                color_scale = if color_scale > 0.0 { color_scale } else { 0.0 };

                color_scale -= d_color;
                d_color *= 0.999995;
                let color = CMAP[(fid % 97) as usize];

                format!("{},{},{},{}\n", p.point.0, p.point.1, color, color_scale)
            })
            .collect::<Vec<String>>(),
    );
    two_d_embeddding
}

async fn update_plot_and_top_k(
    context: TnContext,
    all_points_sorted: Vec<ChunkPoint>,
    top_k_points: Vec<ChunkPoint>,
) {
    let two_d_embeddding = get_plot_data(&all_points_sorted);
    // tracing::info!(target: "tron_app", "two_d_embeddding: {}", two_d_embeddding);

    {
        let two_d_embeddding = BytesMut::from_iter(two_d_embeddding.as_bytes());
        let context_guard = context.write().await;
        let mut stream_data_guard = context_guard.stream_data.write().await;
        let data = stream_data_guard.get_mut("plot_data").unwrap();
        data.1.clear();
        // tracing::info!(target: "tron_app", "length:{}", two_d_embeddding.len());
        data.1.push_back(two_d_embeddding);
        // tracing::info!(target: "tron_app", "stream_data {:?}", data.1[0].len());
    }
    let sse_tx = context.get_sse_tx().await;
    let msg = SseD3PlotTriggerMsg {
        server_event_data: TnServerEventData {
            target: D3PLOT.into(),
            new_state: "ready".into(),
        },
        d3_plot: "re-plot".into(),
    };
    send_sse_msg_to_client(&sse_tx, msg).await;

    let mut docs = HashSet::<String>::new();

    let top_doc = top_k_points
        .iter()
        .flat_map(|p| {
            if docs.contains(&p.chunk.title) {
                None
            } else {
                docs.insert(p.chunk.title.clone());
                let fid = p.chunk.get_fid();
                let color = CMAP[(fid % 97) as usize];
                let item = format!(
                    r#"<div class="py-1" >
                <label for="fid_{fid}" class="px-1" style="color: {color}">{}</label></div>"#,
                    p.chunk.title
                );
                Some(item)
            }
        })
        .collect::<Vec<String>>();
    let top_doc = top_doc.join("\n\n");
    update_and_send_div_with_context(&context, TOP_HIT_DIV, &top_doc).await;

    // let top_chunk = top_k_points
    //     .into_iter()
    //     .map(|p| {
    //         let mut text = String::new();
    //         text.extend(format!("=== CHUNK BGN, TITLE: {}\n", p.chunk.title).chars());
    //         text.push_str(&p.chunk.text);
    //         text.push_str("\n=== CHUNK END \n");
    //         text
    //     })
    //     .collect::<Vec<String>>();
    // let top_chunk = top_chunk.join("\n");

    // {
    //     let context_guard = context.write().await;
    //     let mut asset = context_guard.assets.write().await;
    //     asset.insert("top_k_chunk".into(), TnAsset::String(top_chunk));
    // }
}

fn d3_plot_clicked(context: TnContext, _event: TnEvent, payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        // tracing::info!(target: "tron_app", "event {:?}", event);
        // tracing::info!(target: "tron_app", "payload {:?}", payload);
        let mut all_points = Vec::new();
        let evt_x = serde_json::from_value::<f64>(payload["event_data"]["e_x"].clone()).unwrap();
        let evt_y = serde_json::from_value::<f64>(payload["event_data"]["e_y"].clone()).unwrap();
        // tracing::info!(target: "tron_app", "e_x {:?}", evt_x);
        // tracing::info!(target: "tron_app", "e_y {:?}", evt_y);
        //let filename_to_id = &DOCUMENT_CHUNKS.get().unwrap().filename_to_id;

        let asset_id = {
            let context_guard = context.read().await;
            let assets_guard = context_guard.assets.read().await;
            if let Some(TnAsset::U32(chat_id)) = assets_guard.get("asset_id") {
                *chat_id as i32
            } else {
                panic!("no asset id found")
            }
        };


        let all_doc = get_all_points(asset_id).await;
        all_doc.iter().for_each(|p| {
            let c = p.chunk.clone();
            let two_d_embedding = c.two_d_embedding.unwrap();
            let x = two_d_embedding.0 as f64;
            let y = two_d_embedding.1 as f64;
            let d = OrderedFloat::from((evt_x - x).powi(2) + (evt_y - y).powi(2));
            let point = ChunkPoint {
                d,
                point: (x, y),
                chunk: c.clone(),
            };
            all_points.push(point);
        });
        all_points.sort();
        all_points.reverse();

        let ref_eb_vec = all_points.first().unwrap().chunk.embedding_vec.clone().unwrap();
        let all_points_sorted = vector_query_and_sort_points(asset_id, &ref_eb_vec, None, None).await;
        let top_10: Vec<ChunkPoint> = all_points_sorted[..10].into();

        update_plot_and_top_k(context, all_points_sorted, top_10).await;

        None
    }
}

fn reset_button_clicked(
    context: TnContext,
    event: TnEvent,
    _payload: Value,
) -> TnFutureHTMLResponse {
    tn_future! {
        // tracing::info!(target: "tron_app", "{:?}", event);
        if event.e_trigger != RESET_BUTTON {
            None
        } else {
            {
                let mut two_d_embeddding = "x,y,c,o\n".to_string();
                let asset_id = {
                    let context_guard = context.read().await;
                    let assets_guard = context_guard.assets.read().await;
                    if let Some(TnAsset::U32(chat_id)) = assets_guard.get("asset_id") {
                        *chat_id as i32
                    } else {
                        panic!("no chat id found")
                    }
                };

                let all_chunks = get_all_points(asset_id).await.iter().map(|p| p.chunk.clone()).collect::<Vec<_>>();
                two_d_embeddding.extend([all_chunks
                    .iter()
                    .map(|c| {
                        let fid = c.get_fid();
                        let two_d_embedding = c.two_d_embedding.unwrap();
                        format!(
                            "{},{},{},0.8",
                            two_d_embedding.0,
                            two_d_embedding.1,
                            CMAP[(fid % 97) as usize]
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")]);
                let two_d_embeddding = BytesMut::from_iter(two_d_embeddding.as_bytes());
                {
                    let context_guard = context.write().await;
                    let mut stream_data_guard = context_guard.stream_data.write().await;
                    let data = stream_data_guard.get_mut("plot_data").unwrap();
                    data.1.clear();
                    // tracing::info!(target: "tron_app", "length:{}", two_d_embeddding.len());
                    data.1.push_back(two_d_embeddding);
                    // tracing::info!(target: "tron_app", "stream_data {:?}", data.1[0].len());
                }
            }
            {
                let sse_tx = context.get_sse_tx().await;
                let msg = SseD3PlotTriggerMsg {
                    server_event_data: TnServerEventData {
                        target: D3PLOT.into(),
                        new_state: "ready".into(),
                    },
                    d3_plot: "re-plot".into(),
                };
                send_sse_msg_to_client(&sse_tx, msg).await;
            }
            clean_div_with_context(&context, TOP_HIT_DIV).await;
            None
        }
    }
}
