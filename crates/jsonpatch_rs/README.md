# patch

```
USAGE:
    jsondiffpath-cli patch <ORIGINAL_JSON> <PATCH_JSON>

ARGS:
    <ORIGINAL_JSON>
    <PATCH_JSON>

OPTIONS:
```

<ORIGINAL_JSON> is the base file to apply to the patch

<PATCH_JSON> is the patch file to apply to the base json (followed by JSON-Patch [RFC6902](http://tools.ietf.org/html/rfc6902))

```
jsondiffpatch-cli patch [left.json] [right.json]
```

## e.g.

ORIGINAL_JSON:

```json
{
    "foo": "Hello World",
    "bar": "Unknown"
}
```

PATCH_JSON:

```json
[
    { "op": "replace", "path": "/foo", "value": "new value" },
    { "op": "add", "path": "/baz", "value": "added value" },
    { "op": "remove", "path": "/bar" }
]
```

result:

```json
{
  "baz": "added value",
  "foo": "new value"
}
```