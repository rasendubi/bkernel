
proc main() {.exportc.} =
  var o {.volatile.}: ptr[char] = cast[ptr[char]](0x101f1000)
  var str: cstring = "Hello, world!"
  for i in str:
    o[] = i

  while true:
    discard
