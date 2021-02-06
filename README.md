# taskn

`taskn` is a helper for [Taskwarrior](https://taskwarrior.org/) that makes associating tasks and
notes super, super easy.

## Usage

```bash
$ task
[task next]

ID Age   Tag        Description             Urg
16 -2s   home       fake task                 0

# opens up $EDITOR on a file named after task 16's UUID
$ taskn 16
```

By default, all files opened with taskn are in Markdown.

## Why?

As is the story in a lot [of](https://github.com/crockeo/pj) [my](https://github.com/crockeo/nvim)
[recent](https://github.com/crockeo/orgmode-nvim) [work](https://github.com/crockeo/tasq), I moved
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

I just decided to make the latter the default behavio because...well I just like it more.
