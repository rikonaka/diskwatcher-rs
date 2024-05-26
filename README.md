# diskwatch

Watch the disk files change.

## Usage

### Monitor a Folder

```bash
diskwatcher --watch ./
```

### Or Flush Old Results

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

### Print Files and Folders Change History

```bash
diskwatcher --printdb
```

### Output

```bash
===
path: test/123
md5: 
sha1: 
opt: Added
time: 2024-5-26 21:37:33
===
path: test/123
md5: 
sha1: 
opt: Deleted
time: 2024-5-26 21:37:36
===
path: test/b
md5: d41d8cd98f00b204e9800998ecf8427e
sha1: da39a3ee5e6b4b0d3255bfef95601890afd80709
opt: Deleted
time: 2024-5-26 21:37:41
===
```
