10 REM Mixed BASIC/C80. TEXT @{project::basic_base}. This REM keeps @{buf::fill} literal.
20 PRINT "fill=";@{buf::fill};" cells=";@{buf::cells};" n=";@{buf::COUNT}
30 PRINT "@{buf::fill}"
40 LET R=USR(@{buf::fill},7)
50 PRINT "first=";PEEK(@{buf::cells});" last=";PEEK(@{buf::cells}+@{buf::COUNT}-1)
60 LET R=USR(@{buf::sum},0)
70 PRINT "sum=";R
80 POKE @{buf::cells},1
90 LET R=USR(@{buf::sum},0)
100 PRINT "sum after poke=";R
110 DATA @{buf::fill}:LET R=USR(@{buf::bump},0)
120 PRINT "first after bump=";PEEK(@{buf::cells})
