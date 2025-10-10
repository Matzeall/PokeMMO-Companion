### Base Data based on english dumps consists of:

- monsters.json
- skills.json
- items.json
- locations

### Locale Data consists of simple key(base-data) to value(translation) pairs:

- monsters.json
- locations.json
- locations_pokedex.json
- items.json
- skills.json

### Source Data (the dumps and additional data used to build the Locale/Base-Data) consists of:

- monsters.json (get from pokedex dump)
- skills.json (get from pokedex dump)
- items.json (get from pokedex dump)
- dump_strings (get from normal strings dump)
- additional_translations (optional if you don't mind some english terms, you need to copy the file and manually translate the entries)
  > ![NOTE]
  > More in-depth explanation of the source data is in <a>"./data/source/README.md"</a>

### Example Calls from data-builder dir:

```
../target/debug/data-builder data/source/EN/ data/source/EN/ --out data/locales/EN/
../target/debug/data-builder data/source/EN/ data/source/DE/ --out data/locales/DE/
../target/debug/data-builder data/source/EN/ data/source/IT/ --out data/locales/IT/
../target/debug/data-builder data/source/EN/ data/source/FR/ --out data/locales/FR/
../target/debug/data-builder data/source/EN/ data/source/ES/ --out data/locales/ES/
```

- look at the `build_all_locales.sh` script in the data-builder root dir as a full example
- additionally configure `--base_data_out [DIR]` to also build/copy the base-data
  - if you only want to use the base-data in english, then add `--normal_case_base_data` or `-n` to the end of each call to avoid rewriting the values in lower_case
