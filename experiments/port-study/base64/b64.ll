declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

@__const_b64 = private unnamed_addr constant [64 x i8] [i8 65, i8 66, i8 67, i8 68, i8 69, i8 70, i8 71, i8 72, i8 73, i8 74, i8 75, i8 76, i8 77, i8 78, i8 79, i8 80, i8 81, i8 82, i8 83, i8 84, i8 85, i8 86, i8 87, i8 88, i8 89, i8 90, i8 97, i8 98, i8 99, i8 100, i8 101, i8 102, i8 103, i8 104, i8 105, i8 106, i8 107, i8 108, i8 109, i8 110, i8 111, i8 112, i8 113, i8 114, i8 115, i8 116, i8 117, i8 118, i8 119, i8 120, i8 121, i8 122, i8 48, i8 49, i8 50, i8 51, i8 52, i8 53, i8 54, i8 55, i8 56, i8 57, i8 43, i8 47]

define i64 @encode({ptr, i64} %out, {ptr, i64} %src) nounwind memory(argmem: readwrite, inaccessiblemem: write) {
entry:
  %t1 = extractvalue {ptr, i64} %out, 0
  %t2 = extractvalue {ptr, i64} %out, 1
  %t3 = extractvalue {ptr, i64} %src, 0
  %t4 = extractvalue {ptr, i64} %src, 1
  %t5 = alloca i64
  store i64 %t2, ptr %t5
  %t6 = alloca i64
  store i64 %t4, ptr %t6
  %t7 = load i64, ptr %t5
  %t8 = and i32 2, 63
  %t9 = zext i32 %t8 to i64
  %t10 = lshr i64 %t7, %t9
  %t11 = alloca i64
  store i64 %t10, ptr %t11
  %t12 = load i64, ptr %t11
  %t13 = mul i64 %t12, 3
  %t14 = alloca i64
  store i64 %t13, ptr %t14
  %t15 = load i64, ptr %t6
  %t16 = load i64, ptr %t14
  %t17 = icmp ule i64 %t15, %t16
  %t19 = alloca i1
  store volatile i1 %t17, ptr %t19
  %t20 = load volatile i1, ptr %t19
  br i1 %t20, label %L18, label %trap
L18:
  %t21 = alloca i64
  store i64 %t4, ptr %t21
  %t22 = alloca i64
  store i64 0, ptr %t22
  %t23 = alloca i64
  store i64 0, ptr %t23
  br label %L24
L24:
  %t26 = load i64, ptr %t21
  %t27 = load i64, ptr %t22
  %t28 = sub i64 %t26, %t27
  %t29 = alloca i64
  store i64 %t28, ptr %t29
  %t30 = load i64, ptr %t29
  %t31 = icmp ult i64 %t30, 3
  br i1 %t31, label %L33, label %L34
L33:
  br label %L25
L34:
  br label %L32
L32:
  %t35 = load i64, ptr %t22
  %t36 = add i64 %t35, 1
  %t37 = alloca i64
  store i64 %t36, ptr %t37
  %t38 = load i64, ptr %t22
  %t39 = add i64 %t38, 2
  %t40 = alloca i64
  store i64 %t39, ptr %t40
  %t41 = load i64, ptr %t22
  %t42 = getelementptr i8, ptr %t3, i64 %t41
  %t43 = load i8, ptr %t42
  %t44 = alloca i8
  store i8 %t43, ptr %t44
  %t45 = load i64, ptr %t37
  %t46 = getelementptr i8, ptr %t3, i64 %t45
  %t47 = load i8, ptr %t46
  %t48 = alloca i8
  store i8 %t47, ptr %t48
  %t49 = load i64, ptr %t40
  %t50 = getelementptr i8, ptr %t3, i64 %t49
  %t51 = load i8, ptr %t50
  %t52 = alloca i8
  store i8 %t51, ptr %t52
  %t53 = load i8, ptr %t44
  %t54 = zext i8 %t53 to i32
  %t55 = alloca i32
  store i32 %t54, ptr %t55
  %t56 = load i8, ptr %t48
  %t57 = zext i8 %t56 to i32
  %t58 = alloca i32
  store i32 %t57, ptr %t58
  %t59 = load i8, ptr %t52
  %t60 = zext i8 %t59 to i32
  %t61 = alloca i32
  store i32 %t60, ptr %t61
  %t62 = load i32, ptr %t55
  %t63 = and i32 16, 31
  %t64 = shl i32 %t62, %t63
  %t65 = alloca i32
  store i32 %t64, ptr %t65
  %t66 = load i32, ptr %t58
  %t67 = and i32 8, 31
  %t68 = shl i32 %t66, %t67
  %t69 = alloca i32
  store i32 %t68, ptr %t69
  %t70 = load i32, ptr %t65
  %t71 = load i32, ptr %t69
  %t72 = or i32 %t70, %t71
  %t73 = alloca i32
  store i32 %t72, ptr %t73
  %t74 = load i32, ptr %t73
  %t75 = load i32, ptr %t61
  %t76 = or i32 %t74, %t75
  %t77 = alloca i32
  store i32 %t76, ptr %t77
  %t78 = load i32, ptr %t77
  %t79 = and i32 18, 31
  %t80 = lshr i32 %t78, %t79
  %t81 = alloca i32
  store i32 %t80, ptr %t81
  %t82 = load i32, ptr %t81
  %t83 = and i32 %t82, 63
  %t84 = alloca i32
  store i32 %t83, ptr %t84
  %t85 = load i32, ptr %t84
  %t86 = zext i32 %t85 to i64
  %t87 = alloca i64
  store i64 %t86, ptr %t87
  %t88 = load i32, ptr %t77
  %t89 = and i32 12, 31
  %t90 = lshr i32 %t88, %t89
  %t91 = alloca i32
  store i32 %t90, ptr %t91
  %t92 = load i32, ptr %t91
  %t93 = and i32 %t92, 63
  %t94 = alloca i32
  store i32 %t93, ptr %t94
  %t95 = load i32, ptr %t94
  %t96 = zext i32 %t95 to i64
  %t97 = alloca i64
  store i64 %t96, ptr %t97
  %t98 = load i32, ptr %t77
  %t99 = and i32 6, 31
  %t100 = lshr i32 %t98, %t99
  %t101 = alloca i32
  store i32 %t100, ptr %t101
  %t102 = load i32, ptr %t101
  %t103 = and i32 %t102, 63
  %t104 = alloca i32
  store i32 %t103, ptr %t104
  %t105 = load i32, ptr %t104
  %t106 = zext i32 %t105 to i64
  %t107 = alloca i64
  store i64 %t106, ptr %t107
  %t108 = load i32, ptr %t77
  %t109 = and i32 %t108, 63
  %t110 = alloca i32
  store i32 %t109, ptr %t110
  %t111 = load i32, ptr %t110
  %t112 = zext i32 %t111 to i64
  %t113 = alloca i64
  store i64 %t112, ptr %t113
  %t114 = load i64, ptr %t87
  %t115 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t114
  %t116 = load i8, ptr %t115
  %t117 = alloca i8
  store i8 %t116, ptr %t117
  %t118 = load i64, ptr %t97
  %t119 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t118
  %t120 = load i8, ptr %t119
  %t121 = alloca i8
  store i8 %t120, ptr %t121
  %t122 = load i64, ptr %t107
  %t123 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t122
  %t124 = load i8, ptr %t123
  %t125 = alloca i8
  store i8 %t124, ptr %t125
  %t126 = load i64, ptr %t113
  %t127 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t126
  %t128 = load i8, ptr %t127
  %t129 = alloca i8
  store i8 %t128, ptr %t129
  %t130 = load i64, ptr %t23
  %t131 = add i64 %t130, 1
  %t132 = alloca i64
  store i64 %t131, ptr %t132
  %t133 = load i64, ptr %t23
  %t134 = add i64 %t133, 2
  %t135 = alloca i64
  store i64 %t134, ptr %t135
  %t136 = load i64, ptr %t23
  %t137 = add i64 %t136, 3
  %t138 = alloca i64
  store i64 %t137, ptr %t138
  %t139 = load i8, ptr %t117
  %t140 = load i64, ptr %t23
  %t141 = getelementptr i8, ptr %t1, i64 %t140
  store i8 %t139, ptr %t141
  %t142 = load i8, ptr %t121
  %t143 = load i64, ptr %t132
  %t144 = getelementptr i8, ptr %t1, i64 %t143
  store i8 %t142, ptr %t144
  %t145 = load i8, ptr %t125
  %t146 = load i64, ptr %t135
  %t147 = getelementptr i8, ptr %t1, i64 %t146
  store i8 %t145, ptr %t147
  %t148 = load i8, ptr %t129
  %t149 = load i64, ptr %t138
  %t150 = getelementptr i8, ptr %t1, i64 %t149
  store i8 %t148, ptr %t150
  %t151 = load i64, ptr %t22
  %t152 = add i64 %t151, 3
  store i64 %t152, ptr %t22
  %t153 = load i64, ptr %t23
  %t154 = add i64 %t153, 4
  store i64 %t154, ptr %t23
  br label %L24
L25:
  %t155 = load i64, ptr %t21
  %t156 = load i64, ptr %t22
  %t157 = sub i64 %t155, %t156
  %t158 = alloca i64
  store i64 %t157, ptr %t158
  %t159 = load i64, ptr %t158
  %t160 = icmp eq i64 %t159, 1
  br i1 %t160, label %L162, label %L163
L162:
  %t164 = load i64, ptr %t22
  %t165 = getelementptr i8, ptr %t3, i64 %t164
  %t166 = load i8, ptr %t165
  %t167 = alloca i8
  store i8 %t166, ptr %t167
  %t168 = load i8, ptr %t167
  %t169 = zext i8 %t168 to i32
  %t170 = alloca i32
  store i32 %t169, ptr %t170
  %t171 = load i32, ptr %t170
  %t172 = and i32 2, 31
  %t173 = lshr i32 %t171, %t172
  %t174 = alloca i32
  store i32 %t173, ptr %t174
  %t175 = load i32, ptr %t174
  %t176 = and i32 %t175, 63
  %t177 = alloca i32
  store i32 %t176, ptr %t177
  %t178 = load i32, ptr %t177
  %t179 = zext i32 %t178 to i64
  %t180 = alloca i64
  store i64 %t179, ptr %t180
  %t181 = load i32, ptr %t170
  %t182 = and i32 4, 31
  %t183 = shl i32 %t181, %t182
  %t184 = alloca i32
  store i32 %t183, ptr %t184
  %t185 = load i32, ptr %t184
  %t186 = and i32 %t185, 63
  %t187 = alloca i32
  store i32 %t186, ptr %t187
  %t188 = load i32, ptr %t187
  %t189 = zext i32 %t188 to i64
  %t190 = alloca i64
  store i64 %t189, ptr %t190
  %t191 = load i64, ptr %t180
  %t192 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t191
  %t193 = load i8, ptr %t192
  %t194 = alloca i8
  store i8 %t193, ptr %t194
  %t195 = load i64, ptr %t190
  %t196 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t195
  %t197 = load i8, ptr %t196
  %t198 = alloca i8
  store i8 %t197, ptr %t198
  %t199 = load i64, ptr %t23
  %t200 = add i64 %t199, 1
  %t201 = alloca i64
  store i64 %t200, ptr %t201
  %t202 = load i64, ptr %t23
  %t203 = add i64 %t202, 2
  %t204 = alloca i64
  store i64 %t203, ptr %t204
  %t205 = load i64, ptr %t23
  %t206 = add i64 %t205, 3
  %t207 = alloca i64
  store i64 %t206, ptr %t207
  %t208 = load i8, ptr %t194
  %t209 = load i64, ptr %t23
  %t210 = getelementptr i8, ptr %t1, i64 %t209
  store i8 %t208, ptr %t210
  %t211 = load i8, ptr %t198
  %t212 = load i64, ptr %t201
  %t213 = getelementptr i8, ptr %t1, i64 %t212
  store i8 %t211, ptr %t213
  %t214 = load i64, ptr %t204
  %t215 = getelementptr i8, ptr %t1, i64 %t214
  store i8 61, ptr %t215
  %t216 = load i64, ptr %t207
  %t217 = getelementptr i8, ptr %t1, i64 %t216
  store i8 61, ptr %t217
  %t218 = load i64, ptr %t23
  %t219 = add i64 %t218, 4
  store i64 %t219, ptr %t23
  br label %L161
L163:
  br label %L161
L161:
  %t220 = load i64, ptr %t158
  %t221 = icmp eq i64 %t220, 2
  br i1 %t221, label %L223, label %L224
L223:
  %t225 = load i64, ptr %t22
  %t226 = add i64 %t225, 1
  %t227 = alloca i64
  store i64 %t226, ptr %t227
  %t228 = load i64, ptr %t22
  %t229 = getelementptr i8, ptr %t3, i64 %t228
  %t230 = load i8, ptr %t229
  %t231 = alloca i8
  store i8 %t230, ptr %t231
  %t232 = load i64, ptr %t227
  %t233 = getelementptr i8, ptr %t3, i64 %t232
  %t234 = load i8, ptr %t233
  %t235 = alloca i8
  store i8 %t234, ptr %t235
  %t236 = load i8, ptr %t231
  %t237 = zext i8 %t236 to i32
  %t238 = alloca i32
  store i32 %t237, ptr %t238
  %t239 = load i8, ptr %t235
  %t240 = zext i8 %t239 to i32
  %t241 = alloca i32
  store i32 %t240, ptr %t241
  %t242 = load i32, ptr %t238
  %t243 = and i32 8, 31
  %t244 = shl i32 %t242, %t243
  %t245 = alloca i32
  store i32 %t244, ptr %t245
  %t246 = load i32, ptr %t245
  %t247 = load i32, ptr %t241
  %t248 = or i32 %t246, %t247
  %t249 = alloca i32
  store i32 %t248, ptr %t249
  %t250 = load i32, ptr %t249
  %t251 = and i32 10, 31
  %t252 = lshr i32 %t250, %t251
  %t253 = alloca i32
  store i32 %t252, ptr %t253
  %t254 = load i32, ptr %t253
  %t255 = and i32 %t254, 63
  %t256 = alloca i32
  store i32 %t255, ptr %t256
  %t257 = load i32, ptr %t256
  %t258 = zext i32 %t257 to i64
  %t259 = alloca i64
  store i64 %t258, ptr %t259
  %t260 = load i32, ptr %t249
  %t261 = and i32 4, 31
  %t262 = lshr i32 %t260, %t261
  %t263 = alloca i32
  store i32 %t262, ptr %t263
  %t264 = load i32, ptr %t263
  %t265 = and i32 %t264, 63
  %t266 = alloca i32
  store i32 %t265, ptr %t266
  %t267 = load i32, ptr %t266
  %t268 = zext i32 %t267 to i64
  %t269 = alloca i64
  store i64 %t268, ptr %t269
  %t270 = load i32, ptr %t249
  %t271 = and i32 2, 31
  %t272 = shl i32 %t270, %t271
  %t273 = alloca i32
  store i32 %t272, ptr %t273
  %t274 = load i32, ptr %t273
  %t275 = and i32 %t274, 63
  %t276 = alloca i32
  store i32 %t275, ptr %t276
  %t277 = load i32, ptr %t276
  %t278 = zext i32 %t277 to i64
  %t279 = alloca i64
  store i64 %t278, ptr %t279
  %t280 = load i64, ptr %t259
  %t281 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t280
  %t282 = load i8, ptr %t281
  %t283 = alloca i8
  store i8 %t282, ptr %t283
  %t284 = load i64, ptr %t269
  %t285 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t284
  %t286 = load i8, ptr %t285
  %t287 = alloca i8
  store i8 %t286, ptr %t287
  %t288 = load i64, ptr %t279
  %t289 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t288
  %t290 = load i8, ptr %t289
  %t291 = alloca i8
  store i8 %t290, ptr %t291
  %t292 = load i64, ptr %t23
  %t293 = add i64 %t292, 1
  %t294 = alloca i64
  store i64 %t293, ptr %t294
  %t295 = load i64, ptr %t23
  %t296 = add i64 %t295, 2
  %t297 = alloca i64
  store i64 %t296, ptr %t297
  %t298 = load i64, ptr %t23
  %t299 = add i64 %t298, 3
  %t300 = alloca i64
  store i64 %t299, ptr %t300
  %t301 = load i8, ptr %t283
  %t302 = load i64, ptr %t23
  %t303 = getelementptr i8, ptr %t1, i64 %t302
  store i8 %t301, ptr %t303
  %t304 = load i8, ptr %t287
  %t305 = load i64, ptr %t294
  %t306 = getelementptr i8, ptr %t1, i64 %t305
  store i8 %t304, ptr %t306
  %t307 = load i8, ptr %t291
  %t308 = load i64, ptr %t297
  %t309 = getelementptr i8, ptr %t1, i64 %t308
  store i8 %t307, ptr %t309
  %t310 = load i64, ptr %t300
  %t311 = getelementptr i8, ptr %t1, i64 %t310
  store i8 61, ptr %t311
  %t312 = load i64, ptr %t23
  %t313 = add i64 %t312, 4
  store i64 %t313, ptr %t23
  br label %L222
L224:
  br label %L222
L222:
  %t314 = load i64, ptr %t23
  ret i64 %t314
trap:
  call void @llvm.trap()
  unreachable
}
