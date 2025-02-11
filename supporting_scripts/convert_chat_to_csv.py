"""
Convert chat messages from JSON to CSV.

Usage:
  convert_chat.py <input_file>
  convert_chat.py (-h | --help)

Options:
  -h --help     Show this screen.
  <input_file>  Path to the input JSON file.
"""

import json
import csv
from docopt import docopt

def convert_json_to_csv(input_file):
    # Read the JSON data
    with open(input_file, 'r') as file:
        data = json.load(file)

    messages = data['messages']

    # Prepare data for CSV
    csv_data = []
    user_message = None

    for message in messages:
        if message['role'] == 'user':
            user_message = message
        elif message['role'] == 'bot' and user_message:
            csv_data.append({
                'user_timestamp': user_message['time_stamp'],
                'user_content': user_message['content'],
                'bot_timestamp': message['time_stamp'],
                'bot_content': message['content'],
                'bot_fsm_state': message['fsm_state']
            })
            user_message = None

    # Write to CSV
    csv_filename = 'chat_messages.csv'
    fieldnames = ['user_timestamp', 'user_content', 'bot_timestamp', 'bot_content', 'bot_fsm_state']

    with open(csv_filename, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        for row in csv_data:
            writer.writerow(row)

    print(f"CSV file '{csv_filename}' has been created successfully.")

if __name__ == '__main__':
    arguments = docopt(__doc__)
    input_file = arguments['<input_file>']
    convert_json_to_csv(input_file)