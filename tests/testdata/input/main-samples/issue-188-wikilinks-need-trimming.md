Obsidian trims space before and after the filename in a wikilink target.
These should all be the same:

[[foo]]
[[ foo]]
[[foo ]]
[[    foo    ]]

[[foo|foo]]
[[ foo|foo]]
[[foo |foo]]
[[    foo    |foo]]

[[foo#^abc]]
[[foo#^abc ]]
[[foo#^abc |foo > ^abc]]
[[ foo#^abc ]]
