# zprompt

`zprompt` is a very fast and opinionated zsh prompt. It works for me. Maybe it
works for you. The weirdest and most excellent thing it does is dispatching `git
status` as an async job and rerendering the prompt with a spinner until it
completes, then resolving the indicator to magenta (dirty) or green (clean).

**This lets you have git dirty/clean information in your prompt even in a huge
repo without your prompt taking longer to generate (it takes 5ms for me in our
largest repository).**

The spinner assumes you're using a terminal with true color support because, come
on, it's 2024, what the hell, Apple.

Using `zprompt` is a little weird because of this async job thing.

First off, it expects three environment variables:
* `EXIT_STATUS` - exit status of the last command
* `SHELL_PID` - pid of the shell rendering the prompt
* `PS1_EXEC_NO` - a number that changes each time a new prompt is generated

We use `precmd_functions` to set up `PS1_EXEC_NO`. The async rerender is handled
by sending `SIGALRM` to the shell, which we handle and run `zle reset-prompt` to
redraw the prompt. You don't really have to understand this, just paste this
into your `~/.zshrc`:

```zsh
PROMPT='$(PS1_EXEC_NO=$__ps1_exec_no EXIT_STATUS=$? SHELL_PID=$$ zprompt)'
function __ps1_exec_incr() {
  __ps1_exec_no=$((__ps1_exec_no+1))
}
precmd_functions+=(__ps1_exec_incr)
TRAPALRM() {
  if [[ -n "$WIDGET" ]]; then
    zle reset-prompt
  fi
}
```

## Configuration

`zprompt` has a handful of widgets. The default configuration (`zprompt
"%p%X%s%a%r%n%y %e%P%j "`) is what I use, but you can assemble them in whichever
way you like. You can probably call zprompt multiple times per prompt with
different format strings if you want, but I would really strongly recommend
against having `%a` multiple times per prompt!

* `%p` (path): last component of current working directory
* `%X` (space_if_git): a single space character, if currently in a git repo
* `%s` (stash): a superscript character indictating the number of git stash
  entries, if any
* `%a` (async_data): async git status (just a color code applied to the next
  element)
* `%r` (ref_inf): git ref - branch name or SHA. Uncolored, assumed to be colored
  by async_data
* `%n` (pending): visual indicator for any in-progress cherry-pick merge,
  bisect, or rebase operations.
* `%y` (sync): single character indicating if branch is out of sync with remote
  (or status is unknown)
* `%e` (exit): exit code in red if last command was unsuccessful
* `%P` (prompt): prompt char. % normally, # if root
* `%j` (jobs): number of running jobs (you'll see this if you Ctrl-Z something)

