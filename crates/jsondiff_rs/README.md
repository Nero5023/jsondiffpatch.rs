# diff

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

## e.g.

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
