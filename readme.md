a tool i made to debug/fix the issue of fw image gen leading to a broken fs save index which broke starting firmware version 17.0.0

```
Usage: fix-fs-index-save [OPTIONS] <COMMAND>

Commands:
  print        print contents of an IMKV interpreted as a FS save index
  gen-save     generate a FS save index save from a list of saves to index
  update-save  index supplied saves and add it to an existing index save
  fix-sys      point at a SYSTEM mount where the FS save index save is missing an entry for the NCM content db and hope for the best
  help         Print this message or the help of the given subcommand(s)

Options:
  -t, --tmpdir <TMPDIR>          tempdir to use for extractions/repacks. useful for debugging
      --hactoolnet <HACTOOLNET>  path to hactoolnet executable
  -h, --help                     Print help
```
