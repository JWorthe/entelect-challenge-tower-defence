#!/bin/bash

set -e

REPLAY_FOLDER=$1
OUTPUT_FOLDER=$2

mkdir -p $OUTPUT_FOLDER

for round_folder in $REPLAY_FOLDER/*; do
    round_name=`basename "$round_folder"`
    mkdir -p "$OUTPUT_FOLDER/$round_name"

    player_folders=( "$round_folder"/* )
    player_folder=${player_folders[0]}
    cp "$player_folder/JsonMap.json" "$OUTPUT_FOLDER/$round_name/state.json"
    cp "$player_folder/PlayerCommand.txt" "$OUTPUT_FOLDER/$round_name/PlayerCommand.txt"
    
    opponent_folder=${player_folders[1]}
    cp "$opponent_folder/PlayerCommand.txt" "$OUTPUT_FOLDER/$round_name/OpponentCommand.txt"
done
