
                function fail() { bad=1; exit 1 }
                /^# backend=/ {
                    if (NR!=1 || meta++ || split($0,p," ")!=15 || p[1]!="#" || p[2]!="backend=" backend ||
                        p[3]!="width=" width || p[4]!="shape=" shape || p[5]!="records=" count ||
                        p[6]!~/^bytes=[0-9]+$/ || p[7]!="max_length=" limit || p[8]!="grain=" grain ||
                        p[9]!="chunks=" chunks || p[10]!="seed=" seed || p[11]!="pass=" pass ||
                        p[12]!="reps=" reps || p[13]!~/^clock_pair_min_ns=[0-9]+$/ ||
                        p[14]!="cadence=" cadence || p[15]!="requested_gap_ns=" requested_gap) fail();
                    split(p[6],pair,"="); bytes=pair[2];
                    if (length(expected_bytes) && bytes!=expected_bytes) fail();
                    next
                }
                /^backend\t/ {
                    if (NR!=2 || meta!=1 || header++ ||
                        $0!="backend\twidth\tshape\trecords\tbytes\tmax_length\tgrain\tchunks\tseed\tpass\tcall\tphase\tcall_ns\tuser_us\tsystem_us\tmaxrss_bytes\tvoluntary_switches\tinvoluntary_switches\tcadence\tgap_ns") fail();
                    next
                }
                /^# batch_includes_first=/ {
                    if (header!=1 || seen!=reps+1 || batch++ || capacity || passed || split($0,p," ")!=9 ||
                        p[1]!="#" || p[2]!="batch_includes_first=1" || p[3]!="batch_includes_checks=" checks ||
                        p[4]!~/^batch_ns=[0-9]+$/ || p[5]!~/^user_us=[0-9]+$/ || p[6]!~/^system_us=[0-9]+$/ ||
                        p[7]!~/^maxrss_bytes=[0-9]+$/ || p[8]!~/^voluntary_switches=[0-9]+$/ ||
                        p[9]!~/^involuntary_switches=[0-9]+$/) fail();
                    for(i=4;i<=9;i++) { split(p[i],pair,"="); totals[i]=pair[2]+0; }
                    if (all[13]+gap_sum>totals[4] || all[14]>totals[5] || all[15]>totals[6] ||
                        all_rss>totals[7] || all[17]>totals[8] || all[18]>totals[9]) fail();
                    next
                }
                /^# post_timing_capacity=/ {
                    if (batch!=1 || seen!=reps+1 || capacity++ || passed || split($0,p," ")!=6 ||
                        p[1]!="#" || p[2]!="post_timing_capacity=" width || p[3]!="includes_caller=1" ||
                        p[4]!~/^capacity_waves=[1-3]$/ || p[5]!="explicit_shutdown=" shutdown || p[6]!~/^stop_ns=[0-9]+$/) fail();
                    next
                }
                /^record scheduler benchmark PASS:/ {
                    if (capacity!=1 || passed++ || $0!="record scheduler benchmark PASS: calls=" (reps+1) " outputs=" ((reps+1)*count)) fail();
                    next
                }
                {
                    if (header!=1 || batch || capacity || passed || NF!=20 || $1!=backend || $2!=width || $3!=shape ||
                        $4!=count || $5!=bytes || $6!=limit || $7!=grain || $8!=chunks || $9!=seed || $10!=pass ||
                        $11!=seen || $12!=(seen?"warm":"first") || $19!=cadence ||
                        $20!~/^[0-9]+$/ || (!seen && $20!=0)) fail();
                    for(i=13;i<=18;i++) if($i!~/^[0-9]+$/) fail();
                    for(i=13;i<=18;i++) all[i]+=$i;
                    if(!seen || $16>all_rss)all_rss=$16;
                    gap_sum+=$20;
                    if (seen) {
                        for(i=13;i<=18;i++) sum[i]+=$i;
                        if(seen==1 || $13<minimum)minimum=$13;
                        if(seen==1 || $13>maximum)maximum=$13;
                        if(seen==1 || $16>rss)rss=$16;
                        if(seen==1 || $20<gap_minimum)gap_minimum=$20;
                        if(seen==1 || $20>gap_maximum)gap_maximum=$20;
                    }
                    seen++;
                }
                END {
                    if (bad || meta!=1 || header!=1 || batch!=1 || capacity!=1 || passed!=1 || seen!=reps+1) exit 1;
                    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%d",backend,width,shape,count,bytes,limit,grain,chunks,seed,pass,reps;
                    printf "\t%.3f\t%.0f\t%.0f\t%.3f\t%.3f\t%.3f\t%.0f\t%.0f\t%.0f",sum[13]/reps,minimum,maximum,sum[14]/reps,sum[15]/reps,(sum[14]+sum[15])/reps,sum[17],sum[18],rss;
                    printf "\t%s\t%.0f\t%.3f\t%.0f\t%.0f",cadence,requested_gap,gap_sum/reps,gap_minimum,gap_maximum;
                    printf "\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%.0f\t%d\n",totals[4],totals[5],totals[6],totals[5]+totals[6],totals[7],totals[8],totals[9],checks;
                }
