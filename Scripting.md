# decoder / modifier in hraw

## description

can use luau and python.

## requirement

- python
- numpy <= 1.26.0
  - not supposed, numpy >= 2.0

## installation

```powershell
ps> winget search Python.Python
ps> winget install --id Python.Python.3.xx

PS> pip install numpy==1.26.0
...
```

## usage

### foreign function interface in rust

**<ins>decoder</ins>**
- read file to byte array
- input args : ```pixel index```
  - ```usize```
  - ```x + y * width```
- builtins table / object : ```rawbytes```
- return value : ```i32``` ( ```f32``` / ```f64``` )

| builtins | lua member | py member | type |
| :--      | :--                  | :--                  | :--:             |
| rawbytes | width : number       |                      | readonly field   |
|          | height : number      |                      | readonly field   |
|          |                      | width() -> int       | method           |
|          |                      | height() -> int      | method           |
|          | get(number) : number | get(number) -> int   | method           |
|          | to_table() : table   | to_array() -> list   | method           |
|          | [number] : number    | [any] -> list        | readonly indexer |

**<ins>modifier</ins>**
- input args : ```json Value```
- builtins table / object : ```pixel```
- return value : ```json Value```

| builtins | lua member | py member | type |
| :--      | :--                         | :--                  | :--:             |
| pixel    | width() -> number           | width() -> int       | method           |
|          | height() -> number          | height() -> int      | method           |
|          | get(number,number) : number | get(int,int) -> int  | method           |
|          | set(number,number,number)   | set(int,int,int)     | method           |
|          | to_table() : table          | to_array() -> list   | method           |
|          |                             | to_np() -> numpy     | method           |
|          |                             | from_array(list)     | method           |
|          |                             | from_np(numpy)       | method           |
|          | [number] : number           | [any] -> list        | readonly indexer |

### with lua

#### <ins>detail</ins>

- luau
  - [luau-lang.org](https://luau-lang.org/)
  - [Online Lua Compiler](https://luau-lang.org/demo)
- ~~entry point : ```function(index)```~~
- entry point : ```function main(index)```
  - use global vals

#### <ins>example</ins>

**<ins>decoder</ins>**
```lua
--!strict
bytes = buffer.create(4)                    --bytes : global temporary variable
function main(index)                        --index : pixel index, types is number (64-bit double), 
  local i = index * 4 + 0                   --i     : bytes index, index starts with 0
                                            --      : if use table, ```to_table()```, index starts with 1
  buffer.writeu8(bytes, 3, rawbytes[i + 3]) --write to temporary variable from src
  buffer.writeu8(bytes, 2, rawbytes[i + 2])
  buffer.writeu8(bytes, 1, rawbytes[i + 1])
  buffer.writeu8(bytes, 0, rawbytes[i + 0])
  local dst = bit32.rshift(buffer.readi32(buf, 0), 8)  --8bit bitshift
  return dst
end
```

**<ins>modifier</ins>**
```lua
--!strict
function main(args)
  local dst = 0
  local len = pixel:width() * pixel:height()
  for i = 0, len - 1, 1 do                     -- 0..=size
    dst += pixel[i]
    print(i)
  end
  return { result = dst };
end
```

### with python

#### <ins>detail</ins>

- pyo3
  - No portability because it needs an environment setup.
- entry point : ```def function```
- numpy 2.0 is not supported
  - use numpy <= 1.26.0

- Entry Point : default is ```main```
  - Target can be specified in js
- Args : json to dictionary object
- Pixel References : use ```Pixel```
  - IntelliSense throws undefined error, use ```#type: ignore``` to avoid it.
- Returns : dictionary object (Serialize to Json in rust)



#### <ins>example</ins>

**<ins>decoder</ins>**
```python
def main(index):                         # index : pixel index, ```main``` is reserved word
  i = index * 4                          # i     : bytes index
  dst = bytearray([src[i], src[i+1], src[i+2], src[i+3]])
  return int.from_bytes(dst, 'little', signed=True) >> 8
```

**<ins>modifier</ins>**
```python
def main(arg):
  print(arg['a'])
  len = pixel.width() * pixel.height()
  for i in range(0, len, 1):              # 0..len
    print(i)
  return { 'detail' : 'success' }
```

### grammar

### <ins>luau</ins>

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

**metamethod**
- __add
- __sub
- __mul
- __div
- __mod
- __pow
- __unm   : 単項マイナス演算
- __idiv  : 切り捨て演算
- __band  : 論理積演算
- __bor	  : 論理和演算
- __bxor  : 排他的論理和
- __bnot  : 単項の時。否定
- __shl   : 左シフト
- __shr   : 右シフト
- __concat : ..(連結)
- __len   : #(長さ演算)
- __eq    : ==(等値比較)
- __lt    : < 引数（演算する対象）
- __le    : <=	__ltと__eqを組み合わせた処理
- __index : table[key](添え字アクセス)
- __newindex : table[key] = value
- __call  : function(args)(関数呼び出し)

**Class**

```lua
Coord = {}
Coord.new = function(_x,_y)
              local obj = { x = _x, y = _y }
              obj.pp = function(self) -- newのたびに関数の初期化が走る
                         print(string.format("(%d,%d)", self.x, self.y))
                       end
              return obj 
            end

local instance = Coord.new(1,3)
instance.pp(p)
instance:pp()  -- == p.p(p)
```

**クロージャ**

**other**

```lua
--!strict
                                          -- メモリ余裕あるならここで全部読み出しちゃっていい
function main(index : number)             --number = 64-bit
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

- tableのindexは1 start,
- bufferは引数がoffsetなので0 start

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

### <ins>py</ins>

```python
def main(index):
  i = index * 4
  byte = bytearray(src[i:i+4])
  return int.from_bytes(byte, 'little', signed=True) >> 8
```

```python
import struct
def main(index):
  i = index * 4
  byte = bytearray(src[i:i+4])
  return struct.unpack_from('<i', byte, 0)[0]
```

```python
import numpy as np                  # import library

def main(args):                     # entry point
  print('args :', args)             # {} when args is empty 
  Px = pixel #type: ignore          # call struct in rust
  pixel = Px.get(10, 10)            # set / get pixel data (i32)
  Px.set(10, 10, 255)
  df = Px.to_np()                   # convert to np.ndarray
  Px.from_np(df)                    # convert from np.ndarray
  return { 'detail' : 'successed' } # return dictionary object
```