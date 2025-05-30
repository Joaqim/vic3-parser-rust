# Victoria 3 Definitions Parser

- Uses [logos](https://crates.io/crates/logos) to parse input file to tokens.
- Uses [serde](https://crates.io/crates/serde) to serialize tokens to valid JSON.
- Uses [ariadne](https://crates.io/crates/ariadne) to output fancy diagnostics when failing to parse input.

### Input:
```nix
global_variable = 30

SCOPE_1 = {
    array = { "value1" "value2" value3 } # Mixed usage of quoted and un-quoted strings, all are valid
    integer_array = { 1 2 3 }
    float_array = { 1.2 3.4 }
    number_array = { 1.234 4 0.0 } # Arrays can contain mixed simple values
    object = {
        var_str = "string value"
        var_int = 123
        var_mixed_nest_array = { 1 "test1" test2 4 5 6 7.5 } 
    }
    
    empty_object = { }
    empty_array = { }
    # Since an empty object and an empty array are defined the same,
    # an AST `Empty` token will be to represent any empty array/object,
    # For JSON output, will always be an empty array: `[]`
    
    color = "#ff0000"
    hex_color_array = { x00ff00 "xff0000" "x0000ff" } 
    number = 10
    string_literal = "hello"
    implied_string = world    
    integer_var = 100
    float_var = 100.0
    boolean_var = true
}
```
### Output:
```json
{
  "global_variable": 30,
  "SCOPE_1": {
    "array": [
      "value1",
      "value2",
      "value3"
    ],
    "integer_array": [
      1,
      2,
      3
    ],
    "float_array": [
      1.2,
      3.4
    ],
    "number_array": [
      1.234,
      4,
      0.0
    ],
    "object": {
      "var_str": "string value",
      "var_int": 123,
      "var_mixed_nest_array": [
        1,
        "test1",
        "test2",
        4,
        5,
        6,
        7.5
      ]
    },
    "empty_object": [],
    "empty_array": [],
    "color": "#ff0000",
    "hex_color_array": [
      "x00ff00",
      "xff0000",
      "x0000ff"
    ],
    "number": 10,
    "string_literal": "hello",
    "implied_string": "world",
    "integer_var": 100,
    "float_var": 100.0,
    "boolean_var": true
  }
}
```