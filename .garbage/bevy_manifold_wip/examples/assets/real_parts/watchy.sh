#!/bin/bash

# Copyright (C) 2023 Admix Pty. Ltd. - All Rights Reserved.
# Unauthorized copying of this file, via any medium is strictly prohibited.
# Proprietary and confidential.

for dfm_file in *.dfm; do
    valid_file="${dfm_file%.dfm}.valid"
    if [ ! -e "$valid_file" ]; then
        echo "Missing valid file for: $dfm_file"
    fi
done
