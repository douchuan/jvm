use handlebars::Handlebars;

pub const PART_FIELDS: &str = "
{{#each fields}}
  {{ this }}
{{/each}}";

pub const PART_METHODS: &str = "
{{#each methods as |method|}}
  {{ method.desc ~}}
  {{~#if enable_code}}
    Code:
    {{~#each method.codes}}
      {{this ~}}
    {{/each}}
  {{/if}}
  {{~#if enable_line_number}}
    LineNumberTable:
      {{~#each method.line_number_table}}
        line {{this.line_number}}: {{this.start_pc ~}}
      {{/each}}
  {{/if}}
{{/each}}";

pub const CLASS: &str = "Compiled from \"{{source_file}}\"
{{class_head}} {
  {{~> fields ~}}
  {{~> methods ~}}
}";

pub fn get_engine() -> Handlebars<'static> {
    let mut h = Handlebars::new();
    let _ = h.register_partial("fields", PART_FIELDS);
    let _ = h.register_partial("methods", PART_METHODS);
    h.register_escape_fn(handlebars::no_escape);

    h
}
