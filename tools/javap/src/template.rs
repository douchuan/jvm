use handlebars::Handlebars;

pub const PART_COMPILED_FROM: &str = "Compiled from \"{{source_file}}\"";
pub const PART_SYS_INFO: &str = "
Classfile {{sys_info.class_file}}
  Last modified {{sys_info.last_modified}}; size {{sys_info.size}} bytes
  MD5 checksum {{sys_info.checksum}}
  Compiled from \"{{sys_info.compiled_from}}\"";

pub const PART_FIELDS: &str = "
{{~#each fields}}
  {{ this }}
{{/each}}";

pub const PART_METHODS: &str = "
{{~#each methods as |method|}}
  {{ method.desc }}
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

pub const CLASS: &str = "
{{~#if enable_sys_info}}
{{~> sys_info ~}}
{{~else~}}
{{~> compiled_from ~}}
{{/if}}
{{class_head}} {
  {{~> fields }}
  {{~> methods }}
}";

pub fn get_engine() -> Handlebars<'static> {
    let mut h = Handlebars::new();
    let _ = h.register_partial("compiled_from", PART_COMPILED_FROM);
    let _ = h.register_partial("sys_info", PART_SYS_INFO);
    let _ = h.register_partial("fields", PART_FIELDS);
    let _ = h.register_partial("methods", PART_METHODS);
    h.register_escape_fn(handlebars::no_escape);

    h
}
