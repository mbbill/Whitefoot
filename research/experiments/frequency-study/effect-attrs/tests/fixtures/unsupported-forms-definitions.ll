define i64 @bundle_target(i64 %x) #0 {
  ret i64 %x
}

define i64 @metadata_target(i64 %x) #0 {
  ret i64 %x
}

attributes #0 = { nounwind willreturn memory(none) }
