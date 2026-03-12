# Sheetdump

> Visually output the internal structure of a `Sheet`

## Usage
jvn sheetdump <SHEET_FILE>              # Default output
jvn sheetdump <SHEET_FILE> --no-sort    # No sorting
jvn sheetdump <SHEET_FILE> --no-pretty  # No prettifying

## Tip
You can also use `renderer override` to access the internal structure of a `Sheet`, 
for example:
jvn sheetdump <SHEET_FILE> --renderer json | jq ".mappings"
