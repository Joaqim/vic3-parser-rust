# Victoria 3 Definitions Parser

### Input:
```
SCOPE_1 = {
    array = { "value1" "value2" value3 }
    number_array = { 1 2 3 }
    object = {
        var_str = "string value"
        var_int = 123
        var_mixed_nest_array = { 1 "test1" test2 4 5 6 } 
    }
    color = "#ff0000"
    number = 10
    string_literal = "hello"
    implied_string = world    
    number_as_last_var = 100
}
```
### Output:
```json
{
  "SCOPE_1": {
    "color": "#ff0000",
    "string_literal": "hello",
    "array": [
      "value1",
      "value2",
      "value3"
    ],
    "number_array": [
      1.0,
      2.0,
      3.0
    ],
    "object": {
      "var_mixed_nest_array": [
        1.0,
        "test1",
        "test2",
        4.0,
        5.0,
        6.0
      ],
      "var_int": 123.0,
      "var_str": "string value"
    },
    "number_as_last_var": 100.0,
    "implied_string": "world",
    "number": 10.0
  }
}
```