use std::collections::{HashSet, VecDeque};

use askama::Template;
use bytes::{BufMut, Bytes, BytesMut};
use ordered_float::OrderedFloat;
use serde_json::Value;
use tron_app::{
    send_sse_msg_to_client,
    tron_components::{
        d3_plot::SseD3PlotTriggerMsg, div::update_and_send_div_with_context, tn_future, TnAsset,
        TnComponentBase, TnContext, TnEvent, TnFutureHTMLResponse,
    },
    tron_macro::ComponentBase,
    TnServerEventData,
};

use std::collections::HashMap;
use tron_app::tron_components::*;
use tron_app::tron_macro::*;

use crate::embedding_service::{sort_points, TwoDPoint, DOCUMENT_CHUNKS};

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
}

#[derive(Template)] // this will generate the code...
#[template(path = "asset_space_plot.html", escape = "none")] // using the template in this path, relative                                    // to the `templates` dir in the crate root
pub struct AssetSpacePlotTemplate {
    d3_plot: String,
    top_hit_div: String,
    reset_button: String,
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
            let mut two_d_embeddding = "x,y,c,o\n".to_string();
            let filename_to_id = &DOCUMENT_CHUNKS.get().unwrap().filename_to_id;
            two_d_embeddding.extend([DOCUMENT_CHUNKS
                .get()
                .unwrap()
                .chunks
                .iter()
                .map(|c| {
                    let fid = filename_to_id.get(&c.filename).unwrap();
                    format!(
                        "{},{},{},0.8",
                        c.two_d_embedding.0,
                        c.two_d_embedding.1,
                        CMAP[(fid % 97) as usize]
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")]);
            let two_d_embeddding = BytesMut::from_iter(two_d_embeddding.as_bytes());
            tracing::info!(target: "tron_app 1", "length:{}", two_d_embeddding.len());
    
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

        self.html = AssetSpacePlotTemplate {
            d3_plot: d3_plot_output_html,
            top_hit_div: top_hit_div_output_html,
            reset_button: reset_button_output_html,
        }
        .render()
        .unwrap()
    }

    async fn post_render(&mut self, _ctx: &TnContextBase) {}
}

fn get_plot_data(all_points_sorted: &[TwoDPoint]) -> String {
    let mut color_scale = 1.0;
    let mut d_color = 8.0 * color_scale / (all_points_sorted.len() as f64);

    let mut two_d_embeddding = "x,y,c,o\n".to_string();
    let filename_to_id = &DOCUMENT_CHUNKS.get().unwrap().filename_to_id;
    two_d_embeddding.extend(
        all_points_sorted
            .iter()
            .map(|p| {
                let c = p.chunk;
                let fid = filename_to_id.get(&c.filename).unwrap();

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

async fn update_plot_and_top_k<'a>(
    context: TnContext,
    all_points_sorted: Vec<TwoDPoint<'a>>,
    top_k_points: Vec<TwoDPoint<'a>>,
) {
    let two_d_embeddding = get_plot_data(&all_points_sorted);

    {
        let two_d_embeddding = BytesMut::from_iter(two_d_embeddding.as_bytes());
        let context_guard = context.write().await;
        let mut stream_data_guard = context_guard.stream_data.write().await;
        let data = stream_data_guard.get_mut("plot_data").unwrap();
        data.1.clear();
        tracing::info!(target: "tron_app", "length:{}", two_d_embeddding.len());
        data.1.push_back(two_d_embeddding);
        tracing::info!(target: "tron_app", "stream_data {:?}", data.1[0].len());
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
                let fid = DOCUMENT_CHUNKS.get().unwrap().filename_to_id.get(&p.chunk.filename).unwrap();
                let color = CMAP[(fid % 97) as usize];
                //onchange="console.log(event.target.checked)"
                let item = format!(r#"<div class="py-1" >
                <label for="fid_{fid}" class="px-1" style="color: {color}">{}</label></div>"#, p.chunk.title);
                Some(item)
            }
        })
        .collect::<Vec<String>>();
    let top_doc = top_doc.join("\n\n");
    update_and_send_div_with_context(&context, TOP_HIT_DIV, &top_doc).await;

    let top_chunk = top_k_points
        .into_iter()
        .map(|p| {
            let mut text = String::new();
            text.extend(format!("=== CHUNK BGN, TITLE: {}\n", p.chunk.title).chars());
            text.push_str(&p.chunk.text);
            text.push_str("\n=== CHUNK END \n");
            text
        })
        .collect::<Vec<String>>();
    let top_chunk = top_chunk.join("\n");

    {
        let context_guard = context.write().await;
        let mut asset = context_guard.assets.write().await;
        asset.insert("top_k_chunk".into(), TnAsset::String(top_chunk));
    }
}

fn d3_plot_clicked(context: TnContext, event: TnEvent, payload: Value) -> TnFutureHTMLResponse {
    tn_future! {
        tracing::info!(target: "tron_app", "event {:?}", event);
        tracing::info!(target: "tron_app", "payload {:?}", payload);
        let mut all_points = Vec::new();
        let evt_x = serde_json::from_value::<f64>(payload["event_data"]["e_x"].clone()).unwrap();
        let evt_y = serde_json::from_value::<f64>(payload["event_data"]["e_y"].clone()).unwrap();
        tracing::info!(target: "tron_app", "e_x {:?}", evt_x);
        tracing::info!(target: "tron_app", "e_y {:?}", evt_y);
        //let filename_to_id = &DOCUMENT_CHUNKS.get().unwrap().filename_to_id;
        DOCUMENT_CHUNKS.get().unwrap().chunks.iter().for_each(|c| {
            let x = c.two_d_embedding.0 as f64;
            let y = c.two_d_embedding.1 as f64;
            let d = OrderedFloat::from((evt_x - x).powi(2) + (evt_y - y).powi(2));
            let point = TwoDPoint {
                d,
                point: (x, y),
                chunk: c,
            };
            all_points.push(point);
        });
        all_points.sort();
        all_points.reverse();

        let ref_eb_vec = all_points.first().unwrap().chunk.embedding_vec.clone();
        let all_points_sorted = sort_points(&ref_eb_vec);
        let top_10: Vec<TwoDPoint> = all_points_sorted[..10].into();

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
        tracing::info!(target: "tron_app", "{:?}", event);
        if event.e_trigger != RESET_BUTTON {
            None
        } else {
            {
                let mut two_d_embeddding = "x,y,c,o\n".to_string();
                let filename_to_id = &DOCUMENT_CHUNKS.get().unwrap().filename_to_id;
                two_d_embeddding.extend([DOCUMENT_CHUNKS
                    .get()
                    .unwrap()
                    .chunks
                    .iter()
                    .map(|c| {
                        let fid = filename_to_id.get(&c.filename).unwrap();
                        format!(
                            "{},{},{},0.8",
                            c.two_d_embedding.0,
                            c.two_d_embedding.1,
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
                    tracing::info!(target: "tron_app", "length:{}", two_d_embeddding.len());
                    data.1.push_back(two_d_embeddding);
                    tracing::info!(target: "tron_app", "stream_data {:?}", data.1[0].len());
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


            None
        }
    }
}
