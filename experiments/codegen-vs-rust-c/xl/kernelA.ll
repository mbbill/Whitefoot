declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define i64 @mix_n(i64 %seed, i64 %n) {
entry:
  %t1 = alloca i64
  store i64 %seed, ptr %t1
  %t2 = alloca i64
  store i64 0, ptr %t2
  %t3 = alloca i64
  store i64 0, ptr %t3
  br label %L4
L4:
  %t6 = load i64, ptr %t3
  %t7 = icmp uge i64 %t6, %n
  br i1 %t7, label %L9, label %L10
L9:
  br label %L5
L10:
  br label %L8
L8:
  %t11 = load i64, ptr %t1
  %t12 = add i64 %t11, 11400714819323198485
  store i64 %t12, ptr %t1
  %t13 = load i64, ptr %t1
  %t14 = and i64 30, 63
  %t15 = lshr i64 %t13, %t14
  %t16 = alloca i64
  store i64 %t15, ptr %t16
  %t17 = load i64, ptr %t1
  %t18 = load i64, ptr %t16
  %t19 = xor i64 %t17, %t18
  %t20 = alloca i64
  store i64 %t19, ptr %t20
  %t21 = load i64, ptr %t20
  %t22 = mul i64 %t21, 13787848793156543929
  store i64 %t22, ptr %t1
  %t23 = load i64, ptr %t1
  %t24 = and i64 27, 63
  %t25 = lshr i64 %t23, %t24
  %t26 = alloca i64
  store i64 %t25, ptr %t26
  %t27 = load i64, ptr %t1
  %t28 = load i64, ptr %t26
  %t29 = xor i64 %t27, %t28
  %t30 = alloca i64
  store i64 %t29, ptr %t30
  %t31 = load i64, ptr %t30
  %t32 = mul i64 %t31, 10723151780598845931
  store i64 %t32, ptr %t1
  %t33 = load i64, ptr %t1
  %t34 = and i64 31, 63
  %t35 = lshr i64 %t33, %t34
  %t36 = alloca i64
  store i64 %t35, ptr %t36
  %t37 = load i64, ptr %t1
  %t38 = load i64, ptr %t36
  %t39 = xor i64 %t37, %t38
  store i64 %t39, ptr %t1
  %t40 = load i64, ptr %t2
  %t41 = load i64, ptr %t1
  %t42 = xor i64 %t40, %t41
  store i64 %t42, ptr %t2
  %t43 = load i64, ptr %t3
  %t44 = add i64 %t43, 1
  store i64 %t44, ptr %t3
  br label %L4
L5:
  %t45 = load i64, ptr %t2
  ret i64 %t45
}

define i32 @main() {
entry:
  %t1 = call i64 @mix_n(i64 81985529216486895, i64 200000000)
  %t2 = alloca i64
  store i64 %t1, ptr %t2
  %t3 = load i64, ptr %t2
  %t4 = icmp eq i64 %t3, 7363752762991654310
  br i1 %t4, label %L5, label %trap
L5:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}
