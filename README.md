<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**  *generated with [DocToc](https://github.com/thlorenz/doctoc)*

- [taskn](#taskn)
  - [Usage](#usage)
    - [Options](#options)
  - [Why?](#why)
  - [Contributing](#contributing)
    - [Planned Work](#planned-work)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# taskn

`taskn` is a helper for [Taskwarrior](https://taskwarrior.org/) that makes associating tasks and
notes super, super easy.

## Usage

`taskn` opens up all of the notes exported by `task`
when provided with the given arguments.
For example:

- `taskn 1` will open task 1's note.
- `taskn status:pending` will open all pending tasks' notes.
- `taskn +home` will open all tasks' notes with the tag `home`.

```bash
$ task
[task next]

ID Age   Tag        Description             Urg
16 -2s   home       fake task                 0

# opens up $EDITOR on a file named after task 16's UUID
$ taskn 16
```

By default, all files opened with taskn are in Markdown.

### Options

`--editor <editor>` &mdash;
The editor used to open task notes.
If unset, taskn will attempt to use $EDITOR.
If $EDITOR is also unset, taskn will default to `vi`.

`--file-format <file-format>` &mdash;
The file format used for task notes [default: md].

`--root-dir <root-dir>` &mdash;
The directory in which task notes are placed.
If the directory does not already exist,
taskn will create it [default: ~/.taskn]

## Why?

As is the story in a lot [of](https://github.com/crockeo/pj) [my](https://github.com/crockeo/nvim)
[recent](https://github.com/crockeo/orgmode-nvim) work, I moved
away from [Emacs](https://www.gnu.org/software/emacs/) for performance reasons, but deeply missed
orgmode. I recently started learning Taskwarrior to decouple my task management from my editor, but
I found a frustrating missing feature: there was no way to associate notes with a task out of the
box.

My workflow in org-land leveraged outlines to structure my TODOs _as_ text, and now I couldn't even
write text beside my tasks!

[taskopen](https://github.com/jschlatow/taskopen) promises to solve this problem, but it requires
overhead whenever you want to associate a note by either:

- Explicitly annotating the task with the note you want to open
- Adding a special notation that means "generate a filename for this"

I just decided to make the latter the default behavior because...well I just like it more.

## Contributing

Feel free to contribute! I can't promise I'll check this repo very often, but if you assign a PR to
me I'll get around to it Eventually™. Before committing code make sure you install
[pre-commit](https://pre-commit.com/) and set it up to
[run on commit](https://pre-commit.com/#3-install-the-git-hook-scripts).

If you're interested but don't know what to work on,
please make an Issue with a feature proposal
or peruse the Planned Work section below.

### Planned Work

- Bind to Apple's [EventKit](https://developer.apple.com/documentation/eventkit?language=objc)
  so we can stop making events reminders through osascript.
  Related reading: [bindgen for objective C](https://rust-lang.github.io/rust-bindgen/objc.html)
  And for me, my bindgen usage in [envoy-mobile-sys](https://github.com/crockeo/envoy-mobile-sys)

- More generalized wrapper around taskwarrior
  that reduces the amount I have to type.
  Something that's mostly plaintext but has a
  modal interface to reduce keystrokes

More to come :)
