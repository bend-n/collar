# collar

this crate provides `collect_array` and makes it easy to collect to small stack allocated arrays

```rs
use collar::*;
let Some([ty, path, http]) = request.split(' ').collect_array_checked() else {
    return;
};
```
