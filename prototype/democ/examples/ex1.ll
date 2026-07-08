declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define i32 @sign_of(i32 %x) {
entry:
  %t1 = icmp slt i32 %x, 0
  br i1 %t1, label %L3, label %L4
L3:
  ret i32 0
L4:
  %t5 = icmp eq i32 %x, 0
  br i1 %t5, label %L7, label %L8
L7:
  ret i32 1
L8:
  ret i32 2
}

define i32 @main() {
entry:
  %t1 = alloca i32
  store i32 40, ptr %t1
  %t2 = alloca i32
  %t3 = load i32, ptr %t1
  %t4 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t3, i32 2)
  %t5 = extractvalue {i32, i1} %t4, 0
  %t6 = extractvalue {i32, i1} %t4, 1
  br i1 %t6, label %L9, label %L8
L8:
  store i32 %t5, ptr %t2
  br label %L7
L9:
  ret i32 0
L7:
  %t10 = load i32, ptr %t2
  %t11 = icmp eq i32 %t10, 42
  br i1 %t11, label %L12, label %trap
L12:
  %t13 = load i32, ptr %t2
  %t14 = call i32 @sign_of(i32 %t13)
  %t15 = alloca i32
  store i32 %t14, ptr %t15
  %t16 = load i32, ptr %t15
  %t20 = icmp eq i32 %t16, 0
  br i1 %t20, label %L18, label %L19
L18:
  ret i32 0
L19:
  %t23 = icmp eq i32 %t16, 1
  br i1 %t23, label %L21, label %L22
L21:
  ret i32 0
L22:
  %t26 = icmp eq i32 %t16, 2
  br i1 %t26, label %L24, label %L25
L24:
  %t27 = load i32, ptr %t2
  %t28 = icmp eq i32 %t27, 42
  br i1 %t28, label %L29, label %trap
L29:
  br label %L17
L25:
  unreachable
L17:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}
