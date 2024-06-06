{% set colocated_path = page.colocated_path | as_str %}
```rust
{{ load_data(path=colocated_path ~ rust_file) }}
```