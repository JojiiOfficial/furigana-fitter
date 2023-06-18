# Japanese Furigana Fitter

A rust library, that given a Japanese word, such as `引っ掛けた`, and it's base
furigana in the following format: `[引|ひ]っ[掛|か]ける`, transforms the
latter to have the same conjugation as the given word. For example
`[引|ひ]っ[掛|か]けた`. The library handles cases, where certain kanji parts
aren't given as kanji as well.

# Example

```rust
assert_eq!(
    fit_furigana("引っかけた", "[引|ひ]っ[掛|か]ける"),
    Ok("[引|ひ]っかけた".to_owned())
);
```
