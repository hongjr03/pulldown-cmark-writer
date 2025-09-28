in the first paragraph. This seems like a bug, and it's easier to fix it than
to remain compatible with it <https://github.com/pulldown-cmark/pulldown-cmark/pull/929>.

```````````````````````````````` example
My [cmark-gfm][^c].

My [cmark-gfm][cmark-gfm][^c].

My [cmark-gfm][][^c].

My [cmark-gfm] [^c].

My [cmark-gfm[^c]].

[cmark-gfm]: https://github.com/github/cmark-gfm/blob/1e230827a584ebc9938c3eadc5059c55ef3c9abf/test/extensions.txt#L702

[^c]: cmark-gfm is under the MIT license, so incorporating parts of its
    test suite into pulldown-cmark should be fine.


My [otherlink[^c]].

[otherlink[^c]]: https://github.com/github/cmark-gfm/blob/1e230827a584ebc9938c3eadc5059c55ef3c9abf/test/extensions.txt#L702
