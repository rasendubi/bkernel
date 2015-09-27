{.push stack_trace: off, profiler:off.}

proc rawoutput(s: string) =
  var o {.volatile.} = cast[ptr[char]](0x101f1000)
  for i in s:
    o[] = i

proc panic(s: string) =
  while true:
    discard

{.pop.}
