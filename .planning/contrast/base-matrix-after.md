
### Text ramp × surfaces (fg-4 disabled = exempt)

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| --fg-0 on --bg-0 | `var(--fg-0)` | `var(--bg-0)` | 19.59 | ✅ AAA |
| --fg-0 on --bg-1 | `var(--fg-0)` | `var(--bg-1)` | 19.44 | ✅ AAA |
| --fg-0 on --bg-2 | `var(--fg-0)` | `var(--bg-2)` | 18.98 | ✅ AAA |
| --fg-0 on --bg-3 | `var(--fg-0)` | `var(--bg-3)` | 18.33 | ✅ AAA |
| --fg-0 on --bg-hover | `var(--fg-0)` | `var(--bg-hover)` | 17.70 | ✅ AAA |
| --fg-0 on --bg-selected | `var(--fg-0)` | `var(--bg-selected)` | 16.29 | ✅ AAA |
| --fg-1 on --bg-0 | `var(--fg-1)` | `var(--bg-0)` | 13.60 | ✅ AAA |
| --fg-1 on --bg-1 | `var(--fg-1)` | `var(--bg-1)` | 13.49 | ✅ AAA |
| --fg-1 on --bg-2 | `var(--fg-1)` | `var(--bg-2)` | 13.17 | ✅ AAA |
| --fg-1 on --bg-3 | `var(--fg-1)` | `var(--bg-3)` | 12.72 | ✅ AAA |
| --fg-1 on --bg-hover | `var(--fg-1)` | `var(--bg-hover)` | 12.29 | ✅ AAA |
| --fg-1 on --bg-selected | `var(--fg-1)` | `var(--bg-selected)` | 11.31 | ✅ AAA |
| --fg-2 on --bg-0 | `var(--fg-2)` | `var(--bg-0)` | 8.72 | ✅ AAA |
| --fg-2 on --bg-1 | `var(--fg-2)` | `var(--bg-1)` | 8.65 | ✅ AAA |
| --fg-2 on --bg-2 | `var(--fg-2)` | `var(--bg-2)` | 8.44 | ✅ AAA |
| --fg-2 on --bg-3 | `var(--fg-2)` | `var(--bg-3)` | 8.16 | ✅ AAA |
| --fg-2 on --bg-hover | `var(--fg-2)` | `var(--bg-hover)` | 7.88 | ✅ AAA |
| --fg-2 on --bg-selected | `var(--fg-2)` | `var(--bg-selected)` | 7.25 | ✅ AAA |
| --fg-3 on --bg-0 | `var(--fg-3)` | `var(--bg-0)` | 7.74 | ✅ AAA |
| --fg-3 on --bg-1 | `var(--fg-3)` | `var(--bg-1)` | 7.68 | ✅ AAA |
| --fg-3 on --bg-2 | `var(--fg-3)` | `var(--bg-2)` | 7.50 | ✅ AAA |
| --fg-3 on --bg-3 | `var(--fg-3)` | `var(--bg-3)` | 7.25 | ✅ AAA |
| --fg-3 on --bg-hover | `var(--fg-3)` | `var(--bg-hover)` | 7.00 | 🟡 AA |
| --fg-3 on --bg-selected | `var(--fg-3)` | `var(--bg-selected)` | 6.44 | 🟡 AA |
| --fg-4 on --bg-0 | `var(--fg-4)` | `var(--bg-0)` | 2.06 | ❌ FAIL |
| --fg-4 on --bg-1 | `var(--fg-4)` | `var(--bg-1)` | 2.05 | ❌ FAIL |
| --fg-4 on --bg-2 | `var(--fg-4)` | `var(--bg-2)` | 2.00 | ❌ FAIL |
| --fg-4 on --bg-3 | `var(--fg-4)` | `var(--bg-3)` | 1.93 | ❌ FAIL |
| --fg-4 on --bg-hover | `var(--fg-4)` | `var(--bg-hover)` | 1.86 | ❌ FAIL |
| --fg-4 on --bg-selected | `var(--fg-4)` | `var(--bg-selected)` | 1.72 | ❌ FAIL |

### Semantic & accent as text × surfaces

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| --ok on --bg-0 | `var(--ok)` | `var(--bg-0)` | 9.44 | ✅ AAA |
| --ok on --bg-1 | `var(--ok)` | `var(--bg-1)` | 9.36 | ✅ AAA |
| --ok on --bg-2 | `var(--ok)` | `var(--bg-2)` | 9.14 | ✅ AAA |
| --ok on --bg-3 | `var(--ok)` | `var(--bg-3)` | 8.83 | ✅ AAA |
| --warn on --bg-0 | `var(--warn)` | `var(--bg-0)` | 11.82 | ✅ AAA |
| --warn on --bg-1 | `var(--warn)` | `var(--bg-1)` | 11.73 | ✅ AAA |
| --warn on --bg-2 | `var(--warn)` | `var(--bg-2)` | 11.45 | ✅ AAA |
| --warn on --bg-3 | `var(--warn)` | `var(--bg-3)` | 11.06 | ✅ AAA |
| --err on --bg-0 | `var(--err)` | `var(--bg-0)` | 7.82 | ✅ AAA |
| --err on --bg-1 | `var(--err)` | `var(--bg-1)` | 7.76 | ✅ AAA |
| --err on --bg-2 | `var(--err)` | `var(--bg-2)` | 7.57 | ✅ AAA |
| --err on --bg-3 | `var(--err)` | `var(--bg-3)` | 7.31 | ✅ AAA |
| --info on --bg-0 | `var(--info)` | `var(--bg-0)` | 8.56 | ✅ AAA |
| --info on --bg-1 | `var(--info)` | `var(--bg-1)` | 8.49 | ✅ AAA |
| --info on --bg-2 | `var(--info)` | `var(--bg-2)` | 8.29 | ✅ AAA |
| --info on --bg-3 | `var(--info)` | `var(--bg-3)` | 8.01 | ✅ AAA |
| --accent on --bg-0 | `var(--accent)` | `var(--bg-0)` | 9.63 | ✅ AAA |
| --accent on --bg-1 | `var(--accent)` | `var(--bg-1)` | 9.56 | ✅ AAA |
| --accent on --bg-2 | `var(--accent)` | `var(--bg-2)` | 9.33 | ✅ AAA |
| --accent on --bg-3 | `var(--accent)` | `var(--bg-3)` | 9.01 | ✅ AAA |
| --accent-hi on --bg-0 | `var(--accent-hi)` | `var(--bg-0)` | 12.38 | ✅ AAA |
| --accent-hi on --bg-1 | `var(--accent-hi)` | `var(--bg-1)` | 12.28 | ✅ AAA |
| --accent-hi on --bg-2 | `var(--accent-hi)` | `var(--bg-2)` | 11.99 | ✅ AAA |
| --accent-hi on --bg-3 | `var(--accent-hi)` | `var(--bg-3)` | 11.58 | ✅ AAA |
| --accent-lo on --bg-0 | `var(--accent-lo)` | `var(--bg-0)` | 3.49 | ❌ FAIL |
| --accent-lo on --bg-1 | `var(--accent-lo)` | `var(--bg-1)` | 3.46 | ❌ FAIL |
| --accent-lo on --bg-2 | `var(--accent-lo)` | `var(--bg-2)` | 3.38 | ❌ FAIL |
| --accent-lo on --bg-3 | `var(--accent-lo)` | `var(--bg-3)` | 3.26 | ❌ FAIL |

### Button/fill text

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| accent-fg on accent | `var(--accent-fg)` | `var(--accent)` | 9.22 | ✅ AAA |
| on-accent on accent | `var(--color-on-accent)` | `var(--accent)` | 9.22 | ✅ AAA |
| fg-0 on accent | `var(--fg-0)` | `var(--accent)` | 2.03 | ❌ FAIL |
| fg-0 on err (danger btn) | `var(--fg-0)` | `var(--err)` | 2.51 | ❌ FAIL |
| bg-0 on err (danger btn dark text) | `var(--bg-0)` | `var(--err)` | 7.82 | ✅ AAA |
| fg-0 on ok (success btn) | `var(--fg-0)` | `var(--ok)` | 2.08 | ❌ FAIL |
| bg-0 on ok | `var(--bg-0)` | `var(--ok)` | 9.44 | ✅ AAA |
| fg-0 on warn | `var(--fg-0)` | `var(--warn)` | 1.66 | ❌ FAIL |
| bg-0 on warn | `var(--bg-0)` | `var(--warn)` | 11.82 | ✅ AAA |
| accent-fg on accent-hi | `var(--accent-fg)` | `var(--accent-hi)` | 11.85 | ✅ AAA |

### File-status letters (A/M/D…) over list surfaces

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| new on --bg-0 | `var(--color-status-new)` | `var(--bg-0)` | 9.44 | ✅ AAA |
| new on --bg-1 | `var(--color-status-new)` | `var(--bg-1)` | 9.36 | ✅ AAA |
| new on --bg-hover | `var(--color-status-new)` | `var(--bg-hover)` | 8.53 | ✅ AAA |
| new on --bg-selected | `var(--color-status-new)` | `var(--bg-selected)` | 7.85 | ✅ AAA |
| modified on --bg-0 | `var(--color-status-modified)` | `var(--bg-0)` | 11.82 | ✅ AAA |
| modified on --bg-1 | `var(--color-status-modified)` | `var(--bg-1)` | 11.73 | ✅ AAA |
| modified on --bg-hover | `var(--color-status-modified)` | `var(--bg-hover)` | 10.68 | ✅ AAA |
| modified on --bg-selected | `var(--color-status-modified)` | `var(--bg-selected)` | 9.83 | ✅ AAA |
| deleted on --bg-0 | `var(--color-status-deleted)` | `var(--bg-0)` | 7.82 | ✅ AAA |
| deleted on --bg-1 | `var(--color-status-deleted)` | `var(--bg-1)` | 7.76 | ✅ AAA |
| deleted on --bg-hover | `var(--color-status-deleted)` | `var(--bg-hover)` | 7.06 | ✅ AAA |
| deleted on --bg-selected | `var(--color-status-deleted)` | `var(--bg-selected)` | 6.50 | 🟡 AA |
| renamed on --bg-0 | `var(--color-status-renamed)` | `var(--bg-0)` | 8.56 | ✅ AAA |
| renamed on --bg-1 | `var(--color-status-renamed)` | `var(--bg-1)` | 8.49 | ✅ AAA |
| renamed on --bg-hover | `var(--color-status-renamed)` | `var(--bg-hover)` | 7.73 | ✅ AAA |
| renamed on --bg-selected | `var(--color-status-renamed)` | `var(--bg-selected)` | 7.12 | ✅ AAA |
| typechange on --bg-0 | `var(--color-status-typechange)` | `var(--bg-0)` | 9.53 | ✅ AAA |
| typechange on --bg-1 | `var(--color-status-typechange)` | `var(--bg-1)` | 9.46 | ✅ AAA |
| typechange on --bg-hover | `var(--color-status-typechange)` | `var(--bg-hover)` | 8.62 | ✅ AAA |
| typechange on --bg-selected | `var(--color-status-typechange)` | `var(--bg-selected)` | 7.93 | ✅ AAA |
| conflicted on --bg-0 | `var(--color-status-conflicted)` | `var(--bg-0)` | 11.82 | ✅ AAA |
| conflicted on --bg-1 | `var(--color-status-conflicted)` | `var(--bg-1)` | 11.73 | ✅ AAA |
| conflicted on --bg-hover | `var(--color-status-conflicted)` | `var(--bg-hover)` | 10.68 | ✅ AAA |
| conflicted on --bg-selected | `var(--color-status-conflicted)` | `var(--bg-selected)` | 9.83 | ✅ AAA |

### Graph lane colors × graph surfaces

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| --lane-0 on --bg-1 | `var(--lane-0)` | `var(--bg-1)` | 9.91 | ✅ AAA |
| --lane-0 on --bg-selected | `var(--lane-0)` | `var(--bg-selected)` | 8.31 | ✅ AAA |
| --lane-1 on --bg-1 | `var(--lane-1)` | `var(--bg-1)` | 9.22 | ✅ AAA |
| --lane-1 on --bg-selected | `var(--lane-1)` | `var(--bg-selected)` | 7.73 | ✅ AAA |
| --lane-2 on --bg-1 | `var(--lane-2)` | `var(--bg-1)` | 9.02 | ✅ AAA |
| --lane-2 on --bg-selected | `var(--lane-2)` | `var(--bg-selected)` | 7.56 | ✅ AAA |
| --lane-3 on --bg-1 | `var(--lane-3)` | `var(--bg-1)` | 9.46 | ✅ AAA |
| --lane-3 on --bg-selected | `var(--lane-3)` | `var(--bg-selected)` | 7.93 | ✅ AAA |
| --lane-4 on --bg-1 | `var(--lane-4)` | `var(--bg-1)` | 10.73 | ✅ AAA |
| --lane-4 on --bg-selected | `var(--lane-4)` | `var(--bg-selected)` | 8.99 | ✅ AAA |
| --lane-5 on --bg-1 | `var(--lane-5)` | `var(--bg-1)` | 10.10 | ✅ AAA |
| --lane-5 on --bg-selected | `var(--lane-5)` | `var(--bg-selected)` | 8.47 | ✅ AAA |
| --lane-6 on --bg-1 | `var(--lane-6)` | `var(--bg-1)` | 10.19 | ✅ AAA |
| --lane-6 on --bg-selected | `var(--lane-6)` | `var(--bg-selected)` | 8.54 | ✅ AAA |
| --lane-7 on --bg-1 | `var(--lane-7)` | `var(--bg-1)` | 9.62 | ✅ AAA |
| --lane-7 on --bg-selected | `var(--lane-7)` | `var(--bg-selected)` | 8.06 | ✅ AAA |

### Syntax palette × diff line tints (context / add / del / selected)

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| keyword context | `var(--color-syn-keyword)` | `var(--bg-0)` | 9.15 | ✅ AAA |
| keyword add | `var(--color-syn-keyword)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.07 | ✅ AAA |
| keyword del | `var(--color-syn-keyword)` | `var(--bg-0)` + [var(--diff-del-bg)] | 8.23 | ✅ AAA |
| keyword add-selected | `var(--color-syn-keyword)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.12 | ✅ AAA |
| keyword del-selected | `var(--color-syn-keyword)` | `var(--bg-0)` + [var(--diff-del-hi)] | 7.41 | ✅ AAA |
| string context | `var(--color-syn-string)` | `var(--bg-0)` | 9.17 | ✅ AAA |
| string add | `var(--color-syn-string)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.08 | ✅ AAA |
| string del | `var(--color-syn-string)` | `var(--bg-0)` + [var(--diff-del-bg)] | 8.24 | ✅ AAA |
| string add-selected | `var(--color-syn-string)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.13 | ✅ AAA |
| string del-selected | `var(--color-syn-string)` | `var(--bg-0)` + [var(--diff-del-hi)] | 7.42 | ✅ AAA |
| comment context | `var(--color-syn-comment)` | `var(--bg-0)` | 9.14 | ✅ AAA |
| comment add | `var(--color-syn-comment)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.06 | ✅ AAA |
| comment del | `var(--color-syn-comment)` | `var(--bg-0)` + [var(--diff-del-bg)] | 8.22 | ✅ AAA |
| comment add-selected | `var(--color-syn-comment)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.10 | ✅ AAA |
| comment del-selected | `var(--color-syn-comment)` | `var(--bg-0)` + [var(--diff-del-hi)] | 7.40 | ✅ AAA |
| number context | `var(--color-syn-number)` | `var(--bg-0)` | 12.23 | ✅ AAA |
| number add | `var(--color-syn-number)` | `var(--bg-0)` + [var(--diff-add-bg)] | 10.78 | ✅ AAA |
| number del | `var(--color-syn-number)` | `var(--bg-0)` + [var(--diff-del-bg)] | 10.99 | ✅ AAA |
| number add-selected | `var(--color-syn-number)` | `var(--bg-0)` + [var(--diff-add-hi)] | 9.50 | ✅ AAA |
| number del-selected | `var(--color-syn-number)` | `var(--bg-0)` + [var(--diff-del-hi)] | 9.90 | ✅ AAA |
| type context | `var(--color-syn-type)` | `var(--bg-0)` | 10.19 | ✅ AAA |
| type add | `var(--color-syn-type)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.99 | ✅ AAA |
| type del | `var(--color-syn-type)` | `var(--bg-0)` + [var(--diff-del-bg)] | 9.17 | ✅ AAA |
| type add-selected | `var(--color-syn-type)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.93 | ✅ AAA |
| type del-selected | `var(--color-syn-type)` | `var(--bg-0)` + [var(--diff-del-hi)] | 8.25 | ✅ AAA |
| function context | `var(--color-syn-function)` | `var(--bg-0)` | 14.70 | ✅ AAA |
| function add | `var(--color-syn-function)` | `var(--bg-0)` + [var(--diff-add-bg)] | 12.96 | ✅ AAA |
| function del | `var(--color-syn-function)` | `var(--bg-0)` + [var(--diff-del-bg)] | 13.22 | ✅ AAA |
| function add-selected | `var(--color-syn-function)` | `var(--bg-0)` + [var(--diff-add-hi)] | 11.43 | ✅ AAA |
| function del-selected | `var(--color-syn-function)` | `var(--bg-0)` + [var(--diff-del-hi)] | 11.90 | ✅ AAA |
| variable context | `var(--color-syn-variable)` | `var(--bg-0)` | 13.93 | ✅ AAA |
| variable add | `var(--color-syn-variable)` | `var(--bg-0)` + [var(--diff-add-bg)] | 12.28 | ✅ AAA |
| variable del | `var(--color-syn-variable)` | `var(--bg-0)` + [var(--diff-del-bg)] | 12.53 | ✅ AAA |
| variable add-selected | `var(--color-syn-variable)` | `var(--bg-0)` + [var(--diff-add-hi)] | 10.83 | ✅ AAA |
| variable del-selected | `var(--color-syn-variable)` | `var(--bg-0)` + [var(--diff-del-hi)] | 11.27 | ✅ AAA |
| constant context | `var(--color-syn-constant)` | `var(--bg-0)` | 10.29 | ✅ AAA |
| constant add | `var(--color-syn-constant)` | `var(--bg-0)` + [var(--diff-add-bg)] | 9.07 | ✅ AAA |
| constant del | `var(--color-syn-constant)` | `var(--bg-0)` + [var(--diff-del-bg)] | 9.26 | ✅ AAA |
| constant add-selected | `var(--color-syn-constant)` | `var(--bg-0)` + [var(--diff-add-hi)] | 8.00 | ✅ AAA |
| constant del-selected | `var(--color-syn-constant)` | `var(--bg-0)` + [var(--diff-del-hi)] | 8.33 | ✅ AAA |
| operator context | `var(--color-syn-operator)` | `var(--bg-0)` | 14.02 | ✅ AAA |
| operator add | `var(--color-syn-operator)` | `var(--bg-0)` + [var(--diff-add-bg)] | 12.36 | ✅ AAA |
| operator del | `var(--color-syn-operator)` | `var(--bg-0)` + [var(--diff-del-bg)] | 12.60 | ✅ AAA |
| operator add-selected | `var(--color-syn-operator)` | `var(--bg-0)` + [var(--diff-add-hi)] | 10.90 | ✅ AAA |
| operator del-selected | `var(--color-syn-operator)` | `var(--bg-0)` + [var(--diff-del-hi)] | 11.34 | ✅ AAA |
| punctuation context | `var(--color-syn-punctuation)` | `var(--bg-0)` | 9.15 | ✅ AAA |
| punctuation add | `var(--color-syn-punctuation)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.07 | ✅ AAA |
| punctuation del | `var(--color-syn-punctuation)` | `var(--bg-0)` + [var(--diff-del-bg)] | 8.23 | ✅ AAA |
| punctuation add-selected | `var(--color-syn-punctuation)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.11 | ✅ AAA |
| punctuation del-selected | `var(--color-syn-punctuation)` | `var(--bg-0)` + [var(--diff-del-hi)] | 7.41 | ✅ AAA |
| attribute context | `var(--color-syn-attribute)` | `var(--bg-0)` | 13.93 | ✅ AAA |
| attribute add | `var(--color-syn-attribute)` | `var(--bg-0)` + [var(--diff-add-bg)] | 12.28 | ✅ AAA |
| attribute del | `var(--color-syn-attribute)` | `var(--bg-0)` + [var(--diff-del-bg)] | 12.53 | ✅ AAA |
| attribute add-selected | `var(--color-syn-attribute)` | `var(--bg-0)` + [var(--diff-add-hi)] | 10.83 | ✅ AAA |
| attribute del-selected | `var(--color-syn-attribute)` | `var(--bg-0)` + [var(--diff-del-hi)] | 11.27 | ✅ AAA |
| tag context | `var(--color-syn-tag)` | `var(--bg-0)` | 9.15 | ✅ AAA |
| tag add | `var(--color-syn-tag)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.07 | ✅ AAA |
| tag del | `var(--color-syn-tag)` | `var(--bg-0)` + [var(--diff-del-bg)] | 8.23 | ✅ AAA |
| tag add-selected | `var(--color-syn-tag)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.12 | ✅ AAA |
| tag del-selected | `var(--color-syn-tag)` | `var(--bg-0)` + [var(--diff-del-hi)] | 7.41 | ✅ AAA |
| property context | `var(--color-syn-property)` | `var(--bg-0)` | 13.93 | ✅ AAA |
| property add | `var(--color-syn-property)` | `var(--bg-0)` + [var(--diff-add-bg)] | 12.28 | ✅ AAA |
| property del | `var(--color-syn-property)` | `var(--bg-0)` + [var(--diff-del-bg)] | 12.53 | ✅ AAA |
| property add-selected | `var(--color-syn-property)` | `var(--bg-0)` + [var(--diff-add-hi)] | 10.83 | ✅ AAA |
| property del-selected | `var(--color-syn-property)` | `var(--bg-0)` + [var(--diff-del-hi)] | 11.27 | ✅ AAA |
| regex context | `var(--color-syn-regex)` | `var(--bg-0)` | 9.16 | ✅ AAA |
| regex add | `var(--color-syn-regex)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.08 | ✅ AAA |
| regex del | `var(--color-syn-regex)` | `var(--bg-0)` + [var(--diff-del-bg)] | 8.24 | ✅ AAA |
| regex add-selected | `var(--color-syn-regex)` | `var(--bg-0)` + [var(--diff-add-hi)] | 7.12 | ✅ AAA |
| regex del-selected | `var(--color-syn-regex)` | `var(--bg-0)` + [var(--diff-del-hi)] | 7.41 | ✅ AAA |
| escape context | `var(--color-syn-escape)` | `var(--bg-0)` | 11.09 | ✅ AAA |
| escape add | `var(--color-syn-escape)` | `var(--bg-0)` + [var(--diff-add-bg)] | 9.78 | ✅ AAA |
| escape del | `var(--color-syn-escape)` | `var(--bg-0)` + [var(--diff-del-bg)] | 9.97 | ✅ AAA |
| escape add-selected | `var(--color-syn-escape)` | `var(--bg-0)` + [var(--diff-add-hi)] | 8.62 | ✅ AAA |
| escape del-selected | `var(--color-syn-escape)` | `var(--bg-0)` + [var(--diff-del-hi)] | 8.97 | ✅ AAA |

### Syntax palette × word-emphasis stack (stacked-emphasis — AA allowed)

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| keyword word-add | `var(--color-syn-keyword)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.28 | 🟡 AA |
| keyword word-del | `var(--color-syn-keyword)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 5.73 | 🟡 AA |
| string word-add | `var(--color-syn-string)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.29 | 🟡 AA |
| string word-del | `var(--color-syn-string)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 5.74 | 🟡 AA |
| comment word-add | `var(--color-syn-comment)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.28 | 🟡 AA |
| comment word-del | `var(--color-syn-comment)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 5.73 | 🟡 AA |
| number word-add | `var(--color-syn-number)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 7.06 | ✅ AAA |
| number word-del | `var(--color-syn-number)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 7.66 | ✅ AAA |
| type word-add | `var(--color-syn-type)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.89 | 🟡 AA |
| type word-del | `var(--color-syn-type)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 6.39 | 🟡 AA |
| function word-add | `var(--color-syn-function)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 8.49 | ✅ AAA |
| function word-del | `var(--color-syn-function)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 9.21 | ✅ AAA |
| variable word-add | `var(--color-syn-variable)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 8.04 | ✅ AAA |
| variable word-del | `var(--color-syn-variable)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 8.73 | ✅ AAA |
| constant word-add | `var(--color-syn-constant)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.94 | 🟡 AA |
| constant word-del | `var(--color-syn-constant)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 6.45 | 🟡 AA |
| operator word-add | `var(--color-syn-operator)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 8.09 | ✅ AAA |
| operator word-del | `var(--color-syn-operator)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 8.78 | ✅ AAA |
| punctuation word-add | `var(--color-syn-punctuation)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.28 | 🟡 AA |
| punctuation word-del | `var(--color-syn-punctuation)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 5.73 | 🟡 AA |
| attribute word-add | `var(--color-syn-attribute)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 8.04 | ✅ AAA |
| attribute word-del | `var(--color-syn-attribute)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 8.73 | ✅ AAA |
| tag word-add | `var(--color-syn-tag)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.28 | 🟡 AA |
| tag word-del | `var(--color-syn-tag)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 5.73 | 🟡 AA |
| property word-add | `var(--color-syn-property)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 8.04 | ✅ AAA |
| property word-del | `var(--color-syn-property)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 8.73 | ✅ AAA |
| regex word-add | `var(--color-syn-regex)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 5.29 | 🟡 AA |
| regex word-del | `var(--color-syn-regex)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 5.74 | 🟡 AA |
| escape word-add | `var(--color-syn-escape)` | `var(--bg-0)` + [var(--diff-add-bg), var(--color-diff-word-add-bg)] | 6.40 | 🟡 AA |
| escape word-del | `var(--color-syn-escape)` | `var(--bg-0)` + [var(--diff-del-bg), var(--color-diff-word-delete-bg)] | 6.95 | 🟡 AA |

### Diff plain text (--color-diff-text) over tints

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| diff-text context | `var(--color-diff-text)` | `var(--bg-0)` | 19.59 | ✅ AAA |
| diff-text add | `var(--color-diff-text)` | `var(--bg-0)` + [var(--diff-add-bg)] | 17.27 | ✅ AAA |
| diff-text del | `var(--color-diff-text)` | `var(--bg-0)` + [var(--diff-del-bg)] | 17.61 | ✅ AAA |
| diff-text add-sel | `var(--color-diff-text)` | `var(--bg-0)` + [var(--diff-add-hi)] | 15.23 | ✅ AAA |
| diff-text del-sel | `var(--color-diff-text)` | `var(--bg-0)` + [var(--diff-del-hi)] | 15.85 | ✅ AAA |
| diff-add marker on add | `var(--color-diff-add)` | `var(--bg-0)` + [var(--diff-add-bg)] | 8.32 | ✅ AAA |
| diff-del marker on del | `var(--color-diff-delete)` | `var(--bg-0)` + [var(--diff-del-bg)] | 7.03 | ✅ AAA |

### Text over tinted translucent backgrounds

| element | fg | bg | ratio | verdict |
|---|---|---|---|---|
| fg-2 on muted-bg over bg-1 | `var(--fg-2)` | `var(--bg-1)` + [var(--color-muted-bg)] | 7.82 | ✅ AAA |
| fg-1 on muted-bg over bg-1 | `var(--fg-1)` | `var(--bg-1)` + [var(--color-muted-bg)] | 12.20 | ✅ AAA |
| warn on warning-bg over bg-1 | `var(--warn)` | `var(--bg-1)` + [var(--color-warning-bg)] | 9.31 | ✅ AAA |
| fg-1 on warning-bg over bg-1 | `var(--fg-1)` | `var(--bg-1)` + [var(--color-warning-bg)] | 10.70 | ✅ AAA |
| warn on banner-warn-bg over bg-0 | `var(--warn)` | `var(--bg-0)` + [var(--color-banner-warning-bg)] | 10.74 | ✅ AAA |
| fg-1 on banner-warn-bg over bg-0 | `var(--fg-1)` | `var(--bg-0)` + [var(--color-banner-warning-bg)] | 12.35 | ✅ AAA |
| info on banner-info-bg over bg-0 | `var(--info)` | `var(--bg-0)` + [var(--color-banner-info-bg)] | 7.91 | ✅ AAA |
| fg-1 on banner-info-bg over bg-0 | `var(--fg-1)` | `var(--bg-0)` + [var(--color-banner-info-bg)] | 12.56 | ✅ AAA |
| err on danger-bg over bg-1 | `var(--err)` | `var(--bg-1)` + [var(--color-danger-bg)] | 7.13 | ✅ AAA |
| err on danger-bg-subtle over bg-1 | `var(--err)` | `var(--bg-1)` + [var(--color-danger-bg-subtle)] | 7.21 | ✅ AAA |
| ok on success-bg over bg-1 | `var(--ok)` | `var(--bg-1)` + [var(--color-success-bg)] | 8.49 | ✅ AAA |
| fg-1 on accent-bg over bg-1 | `var(--fg-1)` | `var(--bg-1)` + [var(--color-accent-bg)] | 12.23 | ✅ AAA |
| accent on accent-bg over bg-1 | `var(--accent)` | `var(--bg-1)` + [var(--color-accent-bg)] | 8.66 | ✅ AAA |
| fg-1 on search-current over bg-0 | `var(--fg-1)` | `var(--bg-0)` + [var(--color-search-current)] | 10.11 | ✅ AAA |
| fg-1 on search-match over bg-0 | `var(--fg-1)` | `var(--bg-0)` + [var(--color-search-match)] | 11.79 | ✅ AAA |
| badge-warning on badge-warning-bg over bg-2 | `var(--color-badge-warning)` | `var(--bg-2)` + [var(--color-badge-warning-bg)] | 8.93 | ✅ AAA |


(legend: ✅ AAA ≥7  🟡 AA ≥4.5  ❌ FAIL <4.5 — large-text threshold not applied here)

