# decoder in hraw



## foreign function interface in rust

- read file to byte array
- input args : pixel index
  - usize
  - x + y * width
- global variable : ```src```
  - byte array
- return value : i32 ( f32 / f64 )

## with lua

- luau
  - [luau-lang.org](https://luau-lang.org/)
  - [Online Lua Compiler](https://luau-lang.org/demo)
- entry point : ```function(index)```

**example**
```lua
--!strict
function(index)                        --index : pixel index, types is number (64-bit double), 
  local i = index * 4 + 1;             --i     : bytes index, index starts with 1
  local bytes = buffer.create(4)       --bytes : temporary variable
  buffer.writeu8(bytes, 3, src[i + 3]) --write to temporary variable from src
  buffer.writeu8(bytes, 2, src[i + 2])
  buffer.writeu8(bytes, 1, src[i + 1])
  buffer.writeu8(bytes, 0, src[i + 0])
  local dst = bit32.rshift(buffer.readi32(buf, 0), 8);  --8bit bitshift
  return dst;
end
```

### grammar

**Builtin types**

- nil
- string
- number : 64-bit IEEE754 double precision number
- boolean
- table
- function
- thread
- userdata
- buffer
- any

```lua
--!strict

function(index : number)                  --number = 64-bit
  local i = index * 4 + 1;                --index starts with 1
  local a = bit32.lshift(0xFF, 24)
  local b = bit32.lshift(0xAF, 16)
  local c = bit32.lshift(0x45, 8)
  local d = bit32.lshift(0x32, 0)

  local value = bit32.bor(a, b, c, d)
  print(string.format("0x%016X", value))  -- 0x00000000FFAF4532
  if value > 0x7FFF_FFFF then
    value = -(0xFFFF_FFFF - value1 + 1)  
  end
  print(string.format("0x%016X", value1)) -- 0xFFFFFFFFFFAF4532
  return value;                           -- -5290702
end
```

```lua
--!strict

function(index : number)
  local i = index * 4 + 1;
  local a = bit32.lshift(0xFF, 24)
  local b = bit32.lshift(0xAF, 16)
  local c = bit32.lshift(0x45, 8)
  local d = bit32.lshift(0x32, 0)

  local value = bit32.bor(a, b, c, d)
  value = bit32.lshift(value, 12)
  print(string.format("0x%016X", value2)) -- 0x00000000F4532000 / 32bit bitshift
  value = bit32.rshift(value, 12)
  print(string.format("0x%016X", value2)) -- 0x00000000000F4532 / 32bit unsigned right shift

  return value;
end
```

```lua
--!strict

function(index : number)
  local i = index * 4 + 1;
  local buf = buffer.create(4)            -- create 4byte buffer
  print(string.format("len %d", buffer.len(buf))) 
  buffer.writeu8(buf, 3, 0xFF)            -- write to buffer (little endian)
  buffer.writeu8(buf, 2, 0xAF)
  buffer.writeu8(buf, 1, 0x45)
  buffer.writeu8(buf, 0, 0x32)

  print( string.format("i32 %d", buffer.readi32(buf, 0)) ) -- i32 -5290702         / signed 32-bit integer
  print( string.format("u32 %d", buffer.readu32(buf, 0)) ) -- u32 4289676594       / unsigned 32-bit integer
  print( string.format("0x%X", buffer.readi32(buf, 0)) )   -- 0xFFFFFFFFFFAF4532
  print( string.format("%d", buffer.readf32(buf, 0)) )     -- -9223372036854775808 / 32-bit floating-point number

  return buffer.readi32(buf, 0);                           -- -5290702
end
```

## with python

- pyo3
  - No portability because it needs an environment setup.
- entry point : ```def function```

**example**
```python
def function(index):                     # index : pixel index, ```function``` is reserved word
  i = index * 4                          # i     : bytes index
  dst = bytearray([src[i], src[i+1], src[i+2], src[i+3]])
  return int.from_bytes(dst, 'little', signed=True) >> 8
```

### grammar

```python
def function(index):
  i = index * 4
  dst = bytearray([src[i], src[i+1], src[i+2], src[i+3]])
  return int.from_bytes(dst, 'little', signed=True) >> 8
```

```python
def function(index):
  i = index * 4
  dst = bytearray(src[i:i+4])
  return int.from_bytes(dst, 'little', signed=True) >> 8
```

```python
import struct
def function(index):
  i = index * 4
  bytes = bytearray(src[i:i+4])
  return struct.unpack_from('<i', bytes, 0)[0]
```

```python
import struct
def function(index):
  i = index * 4
  return struct.unpack_from('<i', bytearray(src[i:i+4]), 0)[0]
```