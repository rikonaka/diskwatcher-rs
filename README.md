# diskwatch

Watch the disk files change.

## Usage

### Monitor a folder

```bash
diskwatcher --watch ./
```

### Or flush old results

```bash
diskwatcher --flush --watch ./
```

### Output

```bash
diskwacher runing...
2024-05-26 15:19:07 - File Added: a.txt
2024-05-26 15:19:13 - File Changed: a.txt
2024-05-26 15:19:30 - File Deleted: a.txt
```

### Print files and folders change history

```bash
diskwatcher --printdb
```

### Output

```bash
===
path: test/a
class: File
md5: d41d8cd98f00b204e9800998ecf8427e
sha1: da39a3ee5e6b4b0d3255bfef95601890afd80709
opt: Deleted
time: 2024-5-26 21:37:43
===
path: test/666
class: Folder
md5: 
sha1: 
opt: Deleted
time: 2024-5-26 21:37:55
===
path: test/666
class: Folder
md5: 
sha1: 
opt: Added
time: 2024-5-26 21:37:57
===
```
