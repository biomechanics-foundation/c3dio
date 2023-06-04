Bad C3D sample data from an old Phoenix Technologies Inc. 3D system.

01/07/2003  04:00 PM           338,912 Dance.c3d
01/07/2003  04:14 PM           258,016 golfswing.c3d
01/16/2003  12:08 PM           340,800 Dance1.c3d
01/16/2003  12:12 PM           259,568 golfswing1.c3d

Four sample C3D data files from PhoeniX Technologies - the first two files contain format errors 
that will cause some C3D applications to crash, while the second pair of files contain the same 
data with some of the format errors corrected.  This data is interesting in that it does not use
the +Z orientation as the vertical axis and the two sets of files illustrate the subtle error that
can affect the ability of applications to read C3D data files. 

The first files received (DANCE.C3D and GOLFSWING.C3D) both contain critical errors that prevent
many C3D applications from reading the data correctly - POINT:DATA_START is 0, the POINT:DESCRIPTIONS
and ANALOG:DESCRIPTIONS strings have a dimension length of 0. The ANALOG:OFFSET is stored as a
floating point value and the FORCE_PLATFORMS:USED parameter is missing.

After these errors were reported, Phoenix Technologies modified their C3D application to fix some of
the problems as shown in the two files DANCE1.C3D and GOLFSWING1.C3D but these still contain multiple
parameter errors. 

These files demonstrate that it is not uncommon for manufacturers to fail to understand the C3D file format.