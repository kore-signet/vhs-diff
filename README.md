# vhs-diff
a horrible little diff+patching engine 
works by patching fields as found by their numerical index, using a match table and serde.

## will break if you try using it on:
- enums
- structs with more than 256 fields 
- unit structs
