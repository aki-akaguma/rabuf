TBD
===
Unreleased changes. Release notes have not yet been written.

0.1.3 (2021-11-26)
=====

* rewrite flush() method to be written out in the order of offset.
* adds `buf_overf_rem_all` and `buf_overf_rem_half` to features.
* rewrite the remove strategy at the over limit by the half/all remove.

0.1.2 (2021-11-17)
=====

* adds features: buf_lru, buf_stats

0.1.1 (2021-11-11)
=====

* adds tests
* adds trait and impl: FileSetLen, FileSync, SmallRead, SmallWrite

0.1.0 (2021-11-10)
=====

first commit
