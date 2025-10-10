### Dump source data from PokeMMO client.

To get the source data for a new locale, you first have to download all roms pokemmo needs in this language.
I can recommend "https://myrient.erista.me/files/No-Intro/" for that purpose (using the wget download method works most reliably: `wget -m -np -c -e robots=off -R "index.html*" [URL]`).
When you setup PokeMMO to run with these roms instead, you can dump their data (moddable resources & strings), which will then contain the translations when compared to the english (base-locale) data.

> [!NOTE]
> The client language must always be set to english though, otherwise the dumps will also translate the map keys and the program won't be able to match based on them anymore

> [!NOTE]
> you must rename whichever dump_strings\_\*.xml to just dump_strings.xml. Prefer using the xml of the language you want to translate, but any is fine.

### A full set of source data consists of:

- monsters.json (get from pokedex dump)
- skills.json (get from pokedex dump)
- items.json (get from pokedex dump)
- dump_strings.xml (get from normal strings dump)
- additional_translations.json (optional if you don't mind some english terms, you need to copy the file and manually translate the entries)
