# mdfmt

Formats the tables in your Markdown documents.
Usage: `mdfmt [--in-place] <input> [<output>]`

Before:

```md
| Column A | Column B |
| --- |:-:|
| Apple | Giant Octopus |
| Pear | Pointlessly long item |
```

After:

```md
| Column A | Column B              |
|----------|:---------------------:|
| Apple    | Giant Octopus         |
| Pear     | Pointlessly long item |
```

