# jsondiffpatch.rs



Diff & patch json object.

--------

# Usage

```
jsondiffpatch-cli [Operation] [left.json] [right.json]
```

**Operation** can be *diff* or *patch*

### diff

Compare left.json and right.json 

```
jsondiffpatch-cli diff [left.json] [right.json]
```

#### e.g.

left:

```json
{
    "bar": [
        1, 2, 3
    ],
    "foo": 10
}
```

right:

```json
{
    "bar": [
        2, 3, 4
    ],
    "foo": 11
}
```

diff:

![image-20230124003804101](/Users/nero/Library/Application Support/typora-user-images/image-20230124003804101.png)

### patch

left.json is the base file to apply to the patch

right.json is the patch file to apply to the base json (followed by JSON-Patch [RFC6902](http://tools.ietf.org/html/rfc6902))

```
jsondiffpatch-cli patch [left.json] [right.json]
```

#### e.g.

left.json (base json):

```json
{
    "foo": "Hello World",
    "bar": "Unknown"
}
```

right.json (patch file):

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

