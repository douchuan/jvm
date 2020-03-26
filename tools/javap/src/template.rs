pub const CLASS: &str = "Compiled from \"{{source_file}}\"
{{class_head}} {
    {{#each fields}}
    {{this}}
    {{/each}}

    {{~#each methods as |method| ~}}
        {{~method.desc~}}
    {{#if enable_line_number_table}}
      LineNumberTable:
        {{~#each method.line_number_table}}
          line {{this.line_number}}: {{this.start_pc~}}
        {{/each}}
    {{/if}}
    {{/each}}
}";

pub const ENUM: &str = "Compiled from \"{{source_file}}\"
{{class_head}} {
    {{~#each fields}}
    {{this}}
    {{/each}}
    {{#each methods as |method| ~}}
        {{method.desc}}
      LineNumberTable:\
        {{#each method.line_number_table}}
          line {{this.line_number}}: {{this.start_pc}}\
        {{/each}}\n
    {{/each}}
}";

pub const INTERFACE: &str = "Compiled from \"{{source_file}}\"
{{class_head}} {
{{#each fields}}
    {{this}}
{{/each}}
{{#each methods}}
    {{this}}
{{/each}}
}";