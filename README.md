# sd - s[earch] & d[isplace]

`sd` is an intuitive find & replace CLI.

## The Pitch

Why use it over any existing tools?

**Painless regular expressions**

`sd` uses regex syntax that you already know from JavaScript and Python. Forget
about dealing with quirks of `sed` or `awk` - get productive immediately.

**String-literal mode**

Non-regex find & replace. No more backslashes or remembering which characters
are special and need to be escaped.

**Easy to read, easy to write**

Find & replace expressions are split up, which makes them easy to read and
write. No more messing with unclosed and escaped slashes.

**Smart, common-sense defaults**

Defaults follow common sense and are tailored for typical daily use.

## Comparison to sed

While sed does a whole lot more, `sd` focuses on doing just one thing and doing
it well.

Some cherry-picked examples, where `sd` shines:

- Simpler syntax for replacing all occurrences:
  - sd: `sd before after`
  - sed: `sed s/before/after/g`
- Replace newlines with commas:
  - sd: `sd '\n' ','`
  - sed: `sed ':a;N;$!ba;s/\n/,/g'`
- Extracting stuff out of strings containing slashes:
  - sd: `echo "sample with /path/" | sd '.*(/.*/)' '$1'`
  - sed: use different delimiters every time depending on expression so that
    the command is not completely unreadable
    - `echo "sample with /path/" | sed -E 's/.*(\\/.*\\/)/\1/g'`
    - `echo "sample with /path/" | sed -E 's|.*(/.*/)|\1|g'`
- In place modification of files:
  - sd: `sd before after file.txt`
  - sed: you need to remember to use `-e` or else some platforms will consider
    the next argument to be a backup suffix
    - `sed -i -e 's/before/after/g' file.txt`

## Quick Guide

1. **String-literal mode**. By default, expressions are treated as regex. Use
   `-s` or `--string-mode` to disable regex.

   ```sh
   > echo 'lots((([]))) of special chars' | sd -s '((([])))' ''
   lots of special chars
   ```

2. **Basic regex use** - let's trim some trailing whitespace

   ```sh
   > echo 'lorem ipsum 23   ' | sd '\s+$' ''
   lorem ipsum 23
   ```

3. **Capture groups**

   Indexed capture groups:

   ```sh
   > echo 'cargo +nightly watch' | sd '(\w+)\s+\+(\w+)\s+(\w+)' 'cmd: $1, channel: $2, subcmd: $3'
   cmd: cargo, channel: nightly, subcmd: watch
   ```

   Named capture groups:

   ```sh
   > echo "123.45" | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars dollars and $cents cents'
   123 dollars and 45 cents
   ```

   In the unlikely case you stumble upon ambiguities, resolve them by using
   `${var}` instead of `$var`. Here's an example:

   ```sh
   > echo '123.45' | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars_dollars and $cents_cents'
    and

   > echo '123.45' | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '${dollars}_dollars and ${cents}_cents'
   123_dollars and 45_cents
   ```

4. **Find & replace in a file**

   ```sh
   > sd 'window.fetch' 'fetch' http.js
   ```

   That's it. The file is modified in-place.

   To preview changes:

   ```sh
   > sd -p 'window.fetch' 'fetch' http.js
   ```

5. **Find & replace across project**

   This example uses [fd](https://github.com/sharkdp/fd).

   Good ol' unix philosophy to the rescue.

   ```sh
   fd --type file --exec sd 'from "react"' 'from "preact"'
   ```

   Same, but with backups (consider version control).

   ```bash
   fd --type file --exec cp {} {}.bk \; --exec sd 'from "react"' 'from "preact"'
   ```

### Edge cases

sd will interpret every argument starting with `-` as a (potentially unknown)
flag. The common convention of using `--` to signal the end of flags is
respected:

```bash
$ echo "./hello foo" | sd "foo" "-w"
error: Found argument '-w' which wasn't expected, or isn't valid in this context

USAGE:
    sd [OPTIONS] <find> <replace-with> [files]...

For more information try --help
$ echo "./hello foo" | sd "foo" -- "-w"
./hello -w
$ echo "./hello --foo" | sd -- "--foo" "-w"
./hello -w
```
