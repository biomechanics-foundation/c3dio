A static test gait C3D file with a very large parameter block.  The original C3D standard permitted the
DATA_START value (pointing to the start of the 3D data block) to be a signed INTEGER (values +1 to +127)
thus the maximum DATA_START value expected by many applications is +127. This C3D file has a DATA_START
value of 188 which exceeds the signed limit of +127.  It may crash older applications that interprete
the DATA_START parameter as signed, instead of unsigned.

03/23/2011  11:08 AM         1,349,120 bigparlove.c3d

The maximum DATA_START value for the original "signed" C3D file format is 127 while the maximum DATA_START
value for the current "unsigned" C3D file format is 255.

The file also has two groups with the same name "PROCESSING" - one group contains parameters, the other
group is empty - this seems to be a result of a bug in Vicon Nexus.
