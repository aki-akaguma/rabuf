TBD: rabuf
===
Unreleased changes. Release notes have not yet been written.

0.1.7 (2022-01-07)
=====

* fix some perforamance.

0.1.6 (2021-12-19)
=====

* remove `buf_idx_btreemap` from features.
* fix some bugs of `setup_auto_buf_size()`.
* add name to `struct rabuf` for debugging.
* add `buf_print_hits` to features.

0.1.5 (2021-12-13)
=====

* adds `read_fill_buffer()`.

0.1.4 (2021-12-05)
=====

* adds `buf_pin_zero` to features
* fix bugs: create methods of `struct RaBuf<T>`.
* adds `buf_auto_buf_size` to features

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
