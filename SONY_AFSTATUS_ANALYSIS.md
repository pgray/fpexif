# Sony AFStatus Implementation Guide

## Overview
Sony stores AF (autofocus) status information in multiple ways depending on the camera model. This document explains the structure found in ExifTool's Sony.pm implementation.

## Key Findings

### 1. AFInfo Tag (0x940e)
- **Location**: Main Sony MakerNote
- **Condition**: Used by SLT-/HV/ILCA- models
- **Structure**: Contains a sub-IFD with AF-related data
- **Alternative**: NEX/ILCE models use Tag940e instead

### 2. AFStatus Value Encoding
From `Minolta.pm` (lines 647-658):
```perl
%afStatusInfo = (
    Format => 'int16s',
    # 0=in focus, -32768=out of focus, -ve=front focus, +ve=back focus
    PrintConvColumns => 2,
    PrintConv => {
        0 => 'In Focus',
        -32768 => 'Out of Focus',
        OTHER => sub {
            my ($val, $inv) = @_;
            $inv and $val =~ /([-+]?\d+)/, return $1;
            return $val < 0 ? "Front Focus ($val)" : "Back Focus (+$val)";
        },
    },
);
```

**Value Mapping:**
- `0` = In Focus
- `-32768` (0x8000) = Out of Focus
- Negative values (other than -32768) = Front Focus (distance value)
- Positive values = Back Focus (distance value)

### 3. AFStatus Structure Types

Sony cameras have different AF systems with different numbers of AF points:

#### A. AFStatus15 (15-point AF)
**Tag**: 0x11 in AFInfo (when AFType == 1)
**Format**: int16s[18] - 18 values (15 main points + 3 additional)
**Models**: SLT-A33, A35, A55, DSLR-A560, A580

**Point Layout** (lines 9753-9775):
```
Offset  Name
0x00    AFStatusUpper-left
0x02    AFStatusLeft
0x04    AFStatusLower-left
0x06    AFStatusFarLeft
0x08    AFStatusTopHorizontal
0x0a    AFStatusNearRight
0x0c    AFStatusCenterHorizontal
0x0e    AFStatusNearLeft
0x10    AFStatusBottomHorizontal
0x12    AFStatusTopVertical
0x14    AFStatusCenterVertical
0x16    AFStatusBottomVertical
0x18    AFStatusFarRight
0x1a    AFStatusUpper-right
0x1c    AFStatusRight
0x1e    AFStatusLower-right
0x20    AFStatusUpper-middle
0x22    AFStatusLower-middle
```

#### B. AFStatus19 (19-point AF)
**Tag**: 0x11 in AFInfo (when AFType == 2)
**Format**: int16s[30] - 30 values
**Models**: SLT-A57, A58, A65, A65V, A77, A77V, A99, A99V

**Point Layout** (lines 9778-9812):
```
Offset  Name
0x00    AFStatusUpperFarLeft
0x02    AFStatusUpper-leftHorizontal
0x04    AFStatusFarLeftHorizontal
0x06    AFStatusLeftHorizontal
0x08    AFStatusLowerFarLeft
0x0a    AFStatusLower-leftHorizontal
0x0c    AFStatusUpper-leftVertical
0x0e    AFStatusLeftVertical
0x10    AFStatusLower-leftVertical
0x12    AFStatusFarLeftVertical
0x14    AFStatusTopHorizontal
0x16    AFStatusNearRight
0x18    AFStatusCenterHorizontal
0x1a    AFStatusNearLeft
0x1c    AFStatusBottomHorizontal
0x1e    AFStatusTopVertical
0x20    AFStatusUpper-middle
0x22    AFStatusCenterVertical
0x24    AFStatusLower-middle
0x26    AFStatusBottomVertical
0x28    AFStatusUpperFarRight
0x2a    AFStatusUpper-rightHorizontal
0x2c    AFStatusFarRightHorizontal
0x2e    AFStatusRightHorizontal
0x30    AFStatusLowerFarRight
0x32    AFStatusLower-rightHorizontal
0x34    AFStatusFarRightVertical
0x36    AFStatusUpper-rightVertical
0x38    AFStatusRightVertical
0x3a    AFStatusLower-rightVertical
```

#### C. AFStatus79 (79-point AF)
**Tag**: 0x007d in AFInfo (when AFType == 3)
**Format**: int16s[95] - 95 values (79 points + 15 cross + 1 F2.8)
**Models**: ILCA-68, ILCA-77M2, ILCA-99M2

**Sensor Layout** (from comments lines 9820-9829):
```
                          A5*  A6*  A7*
        B2   B3   B4      B5   B6   B7      B8   B9   B10
  C1    C2   C3   C4      C5   C6   C7      C8   C9   C10  C11
  D1    D2   D3   D4      D5   D6   D7      D8   D9   D10  D11
  E1    E2   E3   E4      E5   E6*  E7      E8   E9   E10  E11
  F1    F2   F3   F4      F5   F6   F7      F8   F9   F10  F11
  G1    G2   G3   G4      G5   G6   G7      G8   G9   G10  G11
        H2   H3   H4      H5   H6   H7      H8   H9   H10
                          I5*  I6*  I7*
```

Points marked with * are cross-sensors (vertical orientation), others are horizontal.

**Point Layout** (lines 9831-9931):
- **Left section** (B4→H4, right to left, top to bottom): Offsets 0x00-0x32
- **Center cross-sensors** (A7→I5, vertical): Offsets 0x34-0x50
- **Center all sensors** (A7→I5, horizontal): Offsets 0x52-0x86
- **Right section** (C11→H8, right to left, top to bottom): Offsets 0x88-0xba
- **Central F2.8 sensor**: Offset 0xbc (E6_Center_F2-8)

### 4. AFStatusActiveSensor
**Location**: Multiple places depending on model
- **Offset 0x04** in AFInfo for SLT models (not ILCA)
- **Offset 0x3b** in AFInfo for ILCA models
- Also appears in CameraInfo (0x0010) at various offsets for different camera families

### 5. Camera-Specific AFStatus in CameraInfo (0x0010)

#### A700/A850/A900 (lines 2800-2823):
```
0x001e  AFStatusActiveSensor
0x0020  AFStatusUpper-left
0x0022  AFStatusLeft
0x0024  AFStatusLower-left
0x0026  AFStatusFarLeft
0x0028  AFStatusBottomAssist-left
0x002a  AFStatusBottom
0x002c  AFStatusBottomAssist-right
0x002e  AFStatusCenter-7
0x0030  AFStatusCenter-horizontal
0x0032  AFStatusCenter-9
0x0034  AFStatusCenter-10
0x0036  AFStatusCenter-11
0x0038  AFStatusCenter-12
0x003a  AFStatusCenter-vertical
0x003c  AFStatusCenter-14
0x003e  AFStatusTopAssist-left
0x0040  AFStatusTop
0x0042  AFStatusTopAssist-right
0x0044  AFStatusFarRight
0x0046  AFStatusUpper-right
0x0048  AFStatusRight
0x004a  AFStatusLower-right
0x004c  AFStatusCenterF2-8
```

#### A200/A230/A290/A300/A330/A350/A380/A390 (lines 2916-2929):
```
0x001b  AFStatusActiveSensor
0x001d  AFStatusTop-right
0x001f  AFStatusBottom-right
0x0021  AFStatusBottom
0x0023  AFStatusMiddleHorizontal
0x0025  AFStatusCenterVertical
0x0027  AFStatusTop
0x0029  AFStatusTop-left
0x002b  AFStatusBottom-left
0x002d  AFStatusLeft
0x002f  AFStatusCenterHorizontal
0x0031  AFStatusRight
```

#### A450/A500/A550/A560/A580 (lines 3080-3130):
```
0x001b  AFStatusActiveSensor (A560/A580 only)
0x001e  AFStatusTop-right (A450/A500/A550)
0x001f  AFStatusBottom-right (A450/A500/A550)
0x0021  AFStatusActiveSensor/Bottom (conditional)
0x0023  AFStatus15 SubDirectory (A560/A580) or AFStatusMiddleHorizontal (A450/A500/A550)
0x0025  AFStatusCenterVertical (A450/A500/A550)
0x0027  AFStatusTop (A450/A500/A550)
0x0029  AFStatusTop-left (A450/A500/A550)
0x002b  AFStatusBottom-left (A450/A500/A550)
0x002d  AFStatusLeft (A450/A500/A550)
0x002f  AFStatusCenterHorizontal (A450/A500/A550)
0x0031  AFStatusRight (A450/A500/A550)
```

## Implementation Strategy for fpexif

1. **Add tag constant**: `SONY_AFINFO = 0x940e`

2. **Parse AFInfo subdirectory** when tag 0x940e is encountered
   - Detect AFType at offset 0x01 (1=15-point, 2=19-point, 3=79-point)
   - Store AFType in parser state for conditional decoding

3. **Implement AFStatus value decoder**:
   ```rust
   pub fn decode_af_status(value: i16) -> String {
       match value {
           0 => "In Focus".to_string(),
           -32768 => "Out of Focus".to_string(),
           v if v < 0 => format!("Front Focus ({})", v),
           v => format!("Back Focus (+{})", v),
       }
   }
   ```

4. **Parse AFStatus arrays** at offset 0x11 (AFStatus15/19) or 0x7d (AFStatus79)
   - Read as int16s array
   - Create individual tags for each position
   - Use decode_af_status for each value

5. **Handle model-specific CameraInfo AFStatus tags**
   - Parse based on camera model detection
   - Use model string to determine which offsets to decode

## Reference Files
- **ExifTool**: `exiftool/lib/Image/ExifTool/Sony.pm`
- **Minolta base**: `exiftool/lib/Image/ExifTool/Minolta.pm` (afStatusInfo structure)

## Testing
Use test files with SLT/ILCA models to verify AFStatus parsing:
- Look for files with AFInfo (0x940e) tag
- Verify AFType detection
- Compare output with `exiftool -a -G1 -s file.ARW | grep AFStatus`
