A very simple program used to run snippets of C code in markdown for my CMSC216 notes. Only works on windows.

To use this program, just run `note-runner <markdown file> <code block>` or `note-runner repl <markdown file>`

The \<code block\> is either the 1-indexed position of the code block (only counts C code blocks) or 'l' followed by the line number of the code block.

When writing your markdown, use the 'lib' tag after if you want this structure/method to be availble to all future code blocks, and if your method codeblock contains a "main" method, use the 'standalone' tag to prevent compiler errors.
