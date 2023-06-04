This file contains Analog and 3D data errors.

01/21/2003  02:50 PM           221,536 sample14.c3d  

A single C3D file that contains 3D and analog data that illustrates a loss of 3D data and analog data 
synchronization - in this example the analog data has been written to the file incorrectly as can be 
clearly seen when looking at the force plate Fz channels.

The errors appear to result from blocks of analog data being writen out of order into the analog data
section.  Note that although the errors affect all analog channels, they are only really visible in 
the Fz channels which clearly show the data transposition errors.