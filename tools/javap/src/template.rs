use handlebars::Handlebars;

pub const PART_COMPILED_FROM: &str = "Compiled from \"{{source_file}}\"";
pub const PART_SYS_INFO: &str = "
Classfile {{sys_info.class_file}}
  Last modified {{sys_info.last_modified}}; size {{sys_info.size}} bytes
  MD5 checksum {{sys_info.checksum}}
  Compiled from \"{{sys_info.compiled_from}}\"";

pub const PART_FIELDS: &str = "
{{~#each fields as |field|}}
  {{ desc }}
  {{~#if enable_inner_signature}}
    descriptor: {{signature~}}
  {{/if}}
{{/each}}";

pub const PART_METHODS: &str = "
{{~#each methods}}
  {{ desc }}
  {{~#if enable_signature}}
    descriptor: {{signature~}}
  {{/if}}
  {{~#if enable_flags}}
    flags: {{flags~}}
  {{/if}}
  {{~#if enable_code}}
    Code:
    {{~#if code.enable_verbose}}
      stack={{code.max_stack}}, locals={{code.max_locals}}, args_size={{code.args_size~}}
    {{/if}}
    {{~#each code.codes}}
      {{this ~}}
    {{/each}}
  {{/if}}
  {{~#if has_ex_table}}
    Exception table:
    {{~#each ex_table}}
      {{this ~}}
    {{/each}}
  {{/if}}
  {{~#if enable_line_number}}
    LineNumberTable:
      {{~#each line_number_table}}
        line {{line_number}}: {{start_pc ~}}
      {{/each}}
  {{/if}}
  {{~#if enable_throws}}
    Exceptions:
      throws {{throws}}
  {{/if}}
  {{~#if enable_stack_map}}
    StackMapTable: number_of_entries = {{stack_map_table.number_of_entries}}
      {{~#each stack_map_table.frames}}
        {{desc}}
        {{~#each items}}
          {{this ~}}
        {{/each~}}
      {{/each}}
  {{/if}}
{{/each}}";

pub const PART_CP: &str = "
Constant pool:
{{~#each cp}}
{{this ~}}
{{/each}}
";

// pub const PART_STACK_MAP_TABLE: &str = "
// StackMapTable: number_of_entries = {{stack_map_table.number_of_entries}}
//   {{~#each stack_map_table.frames}}
//     {{desc}}
//     {{~#each items}}
//       {{this ~}}
//     {{/each}}
//   {{/each}}
// ";

pub const CLASS: &str = "
{{~#if enable_sys_info}}
{{~> sys_info ~}}
{{~else~}}
{{~> compiled_from}}
{{/if}}
{{~#if enable_verbose }}
{{class_head}}
  minor version: {{version.minor}}
  major version: {{version.major}}
  flags: {{flags}}
{{~> constant_pool ~}}
{
{{~else~}}
{{class_head}} {
{{/if}}
  {{~> fields }}
  {{~> methods }}
}";

pub fn get_engine() -> Handlebars<'static> {
    let mut h = Handlebars::new();
    let _ = h.register_partial("compiled_from", PART_COMPILED_FROM);
    let _ = h.register_partial("sys_info", PART_SYS_INFO);
    let _ = h.register_partial("fields", PART_FIELDS);
    let _ = h.register_partial("methods", PART_METHODS);
    let _ = h.register_partial("constant_pool", PART_CP);
    // let _ = h.register_partial("stack_map_table", PART_STACK_MAP_TABLE);
    h.register_escape_fn(handlebars::no_escape);

    h
}
