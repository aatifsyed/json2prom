# json2prom
You have Prometheus running, and you want to scrape an endpoint which exposes JSON.  
This program adapts from that endpoint to Prometheus' exposition format.  

The following JSON...
```
{
    "foo": 12
    "bar": true
    "baz": "stuff"
    "nested": {"nested_stuff_ignored": true}
}
```
...Is adapted into this...
```
foo 12
bar 1
```
- Booleans are cast to 0 (false) or 1 (true)
- Non-numeric data is discarded
- Only the top level keys are exposed

The eventual endpoint MUST be described using an `X-Target` header.  
The exposed JSON MAY be reshaped into a mapping using the optional `X-JQ` header.  