import struct
path = 'test3.raw'

with open(path, 'wb') as f:
  for y in range(0, 3, 1):
    for x in range(0, 3, 1):
      byte = struct.pack('<i', x + y * 3 + 10) # little + signed int
      f.write(byte)

src = [1,0,0,0,5,0,0,0,9,10]
i = 1
dst = struct.unpack_from('<i', bytearray(src[i*4:(i+1)*4]), 0)[0]
print(dst)