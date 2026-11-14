10 REM C80 self-check. selfcheck.c80 lists what each bit of FAILED checks.
20 LET R=USR(@{selfcheck::run},@{selfcheck::PASSES})
30 PRINT "fails=";R;" passes=";PEEK(@{selfcheck::passes})
40 FOR I=0 TO 5
50 PRINT PEEK(@{selfcheck::failed}+I);
60 NEXT I
70 PRINT
80 POKE @{selfcheck::basic_done},1
