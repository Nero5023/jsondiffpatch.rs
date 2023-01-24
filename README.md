# jsondiffpatch.rs



Diff & patch json object.

--------

# Usage

```
USAGE:
    jsondiffpath-cli <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    diff     diff two json file
    help     Print this message or the help of the given subcommand(s)
    patch    patch a json object with a patch document
```

### diff

```
diff two json file

USAGE:
    jsondiffpath-cli diff <LEFT_JSON> <RIGHT_JSON>

ARGS:
    <LEFT_JSON>
    <RIGHT_JSON>

OPTIONS:
    -h, --help    Print help information
```

#### e.g.

LEFT_JSON:

```json
{
    "bar": [
        1, 2, 3
    ],
    "foo": 10
}
```

RIGHT_JSON:

```json
{
    "bar": [
        2, 3, 4
    ],
    "foo": 11
}
```

diff:

![diff_example.png](/Users/nero/local_dev/self_project/jd-rs/imgs/diff_example.png)

### patch

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

#### e.g.

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

