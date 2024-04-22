#!/bin/bash

# https://stackoverflow.com/questions/59895/how-do-i-get-the-directory-where-a-bash-script-is-located-from-within-the-script
cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"

# must go to this directory to get the .env file
cd api_fetcher
./target/release/api_fetcher

cd ..
source .venv/bin/activate
python3 -m display_controller
deactivate