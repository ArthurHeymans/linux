/*
 * Ghidra decompiler export for xr819-annotated / xr819-tcm.bin
 * Functions: 25
 * Reference pseudocode; not buildable source.
 */


/* ======================================================================
 * fff00000  vec_reset
 * ====================================================================== */

void vec_reset(undefined4 param_1,undefined4 param_2)

{
  coproc_moveto_Control(DAT_fff000b0);
                    /* WARNING: Could not recover jumptable at 0xfff00010. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_fff000b4)(DAT_fff000b4,param_2,DAT_fff000b0);
  return;
}



/* ======================================================================
 * fff00004  vec_undef_instr
 * ====================================================================== */

void vec_undef_instr(undefined4 param_1,undefined4 param_2)

{
  coproc_moveto_Control(DAT_fff000b0);
                    /* WARNING: Could not recover jumptable at 0xfff00010. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_fff000b4)(DAT_fff000b4,param_2,DAT_fff000b0);
  return;
}



/* ======================================================================
 * fff00008  vec_swi
 * ====================================================================== */

void vec_swi(undefined4 param_1,undefined4 param_2,undefined4 param_3)

{
  coproc_moveto_Control(param_3);
                    /* WARNING: Could not recover jumptable at 0xfff00010. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_fff000b4)();
  return;
}



/* ======================================================================
 * fff0000c  vec_prefetch_abort
 * ====================================================================== */

void vec_prefetch_abort(void)

{
                    /* WARNING: Could not recover jumptable at 0xfff00010. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*DAT_fff000b4)();
  return;
}



/* ======================================================================
 * fff00010  vec_data_abort
 * ====================================================================== */

void vec_data_abort(code *UNRECOVERED_JUMPTABLE)

{
                    /* WARNING: Could not recover jumptable at 0xfff00010. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*UNRECOVERED_JUMPTABLE)();
  return;
}



/* ======================================================================
 * fff000c8  mib_set_vif_flag_1000000
 * ====================================================================== */

void mib_set_vif_flag_1000000(int param_1)

{
  int iVar1;
  uint uVar2;
  
  if (*(char *)(param_1 + 4) == '\x01') {
    iVar1 = (uint)*(byte *)(DAT_fff004b0 + 10) * 0x3b0 + DAT_fff004b0;
    uVar2 = *(uint *)(iVar1 + 0x1c);
    if ((int)(uVar2 << 0xd) < 0) {
      *(uint *)(iVar1 + 0x1c) = uVar2 | 0x1000000;
    }
  }
  return;
}



/* ======================================================================
 * fff000ec  mib_write_dispatch
 * ====================================================================== */

/* WARNING: Control flow encountered bad instruction data */
/* WARNING: Removing unreachable block (ram,0xfff0016a) */
/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */
/* mib_write_dispatch(u16 *req) -- WSM_WRITE_MIB (0x0006) handler, mib.c.
   Called from wsm_h_06_write_mib at 0x00010E86.
   Returns 0 on success, 2 = "MIB not supported / bad value" (0xFFF0064C:
   mov r6,#2).  Rejects immediately if if_id > 1.
   
   Dispatch, in the order the code tests (r2 = MIB id):
     id <  0x0007 : switch8 @0xFFF0015E, table 0xFFF00162, index = id
     id == 0x0007 : DOT11_RTS_THRESHOLD
     0x0007 < id < 0x1002 : explicit compares for 0x0008, 0x0009, 0x000A,
                    0x000B, then id-0x1000 for 0x1000 / 0x1001; all else reject
     id == 0x1002 : TEMPLATE_FRAME (type < 8, length checked against 0x0001B380)
     0x1002 < id < 0x1013 : switch8 @0xFFF001AC, table 0xFFF001B0,
                    index = id - 0x1002
     id == 0x1013 : SET_UAPSD_INFORMATION (8-byte copy)
     id >  0x1013 : compare ladder on r0 = id - 0x1013, and for r0 < 0x12 a
                    third switch8 @0xFFF001CE, table 0xFFF001D2,
                    index = id - 0x1015  (note the `subs r0,r0,#2` at
                    0xFFF001CA -- the index base is 0x1015, NOT 0x1013)
   
   COMPLETE set of writable MIBs (55):
     0x0001 0x0002 0x0003 0x0004 0x0005 0x0006 0x0007 0x0008 0x0009 0x000A
     0x000B 0x1000 0x1001 0x1002 0x1003 0x1004 0x1005 0x1006 0x1007 0x1009
     0x100B 0x100E 0x100F 0x1010 0x1011 0x1012 0x1013 0x1015 0x1016 0x1017
     0x1019 0x101A 0x101B 0x101C 0x101D 0x101E 0x1020 0x1021 0x1022 0x1024
     0x1025 0x1026 0x1027 0x1028 0x1030 0x1031 0x1032 0x1033 0x1034 0x1035
     0x1039 0x1045 0x1046 0x1050 0x1072
   
   Explicitly REJECTED writes worth knowing:
     0x0000 DOT11_STATION_ID  (set the MAC via WSM_CONFIGURATION 0x0009 instead)
     0x000C SET_TPA_PARAM     -- TPA is absent from this build; 0x1041
                                 TPA_DEBUG_INFO is not readable either
     0x1008, 0x1014           -- no such MIB
     0x100A STATISTICS_TABLE, 0x100C COUNTERS_TABLE, 0x100D BLOCK_ACK_INFO,
     0x101F GRP_SEQ_COUNTER, 0x1023 TSF_COUNTER  -- all read-only
     0x1018 P2P_FIND_INFO     -- matches commands 0x0019/0x001A being
                                 unsupported: P2P find is absent entirely
     0x1036 0x1037 0x1038 0x1040 0x1041 0x1042 0x1043 -- read-only telemetry
   No MIB that either driver actually writes is rejected.
   
   THROUGHPUT-RELEVANT: 0x000A SET_AMPDU_NUM (inline at 0xFFF0017E) caps
   A-MPDU aggregation at 16 subframes:
       ampdu_num = req[2];
       if (ampdu_num <= 0x10) flag[link] = 1;
       else { ampdu_num = 0x10; flag[link] = 0; }
   Neither driver ever writes it, so the firmware runs on its power-on default.
   
   0x1039 BACKOFF_CTRL writes 3 words, clamping the third to 0x400.
   0x0009 RW_FW_REG is an arbitrary memory write -- see mib_read_dispatch. */

undefined4 mib_write_dispatch(ushort *param_1)

{
  char cVar1;
  ushort uVar2;
  undefined *puVar3;
  undefined4 *puVar4;
  int iVar5;
  int iVar6;
  int iVar7;
  undefined4 uVar8;
  ushort *puVar9;
  uint uVar10;
  undefined4 *puVar11;
  int iVar12;
  int iVar13;
  char *pcVar14;
  uint uVar15;
  byte local_34;
  undefined1 local_33;
  ushort local_32;
  ushort *local_30;
  uint local_2c;
  int local_24;
  int local_20;
  int local_1c;
  int local_18;
  
  uVar10 = (uint)*(byte *)(DAT_fff004b0 + 10);
  if (1 < uVar10) {
    return 2;
  }
  iVar5 = 0x3b;
  iVar13 = DAT_fff004b0;
  local_2c = uVar10;
  do {
    puVar4 = DAT_fff004dc;
    puVar11 = DAT_fff004c8;
    iVar12 = DAT_fff004c4;
    puVar3 = PTR_g_tcm_state_fff004bc;
    iVar13 = uVar10 * iVar5 * 0x10 + iVar13;
    uVar10 = (uint)*param_1;
    local_18 = (uint)*(byte *)(DAT_fff004b0 + 10) * 0x98 + DAT_fff004b8;
    pcVar14 = (char *)(iVar13 + 0x18);
    iVar6 = (uint)*(byte *)(DAT_fff004b0 + 10) * 0x3b0 + DAT_fff004b0;
    local_1c = iVar6 + 0x1d8;
    local_20 = iVar6 + 0x398;
    iVar7 = iVar6 + 0x118;
    iVar5 = uVar10 - DAT_fff004b4;
    if (uVar10 == DAT_fff004b4) {
      func_0x0001ae10(iVar13 + 0x130,param_1 + 2,8);
      func_0x0001325a(local_2c);
      return 0;
    }
    if ((int)DAT_fff004b4 <= (int)uVar10) {
      if (iVar5 == 0x12) {
        *(ushort *)(iVar6 + 0x1e0) = param_1[2] * 5;
        return 0;
      }
      if (iVar5 < 0x13) {
        func_0x0001b160();
                    /* WARNING: Bad instruction - Truncating control flow here */
        halt_baddata();
      }
      local_24 = DAT_fff004c4;
      if (iVar5 == 0x21) {
        uVar8 = 0x14;
        iVar13 = 0x390;
      }
      else {
        if (0x21 < iVar5) {
          if (iVar5 == 0x33) {
            if ((param_1[2] & 1) != 0) {
              return 2;
            }
            iVar13 = local_2c * 6 + DAT_fff00914;
            *(ushort *)(iVar13 + 0x454) = param_1[2];
            *(ushort *)(iVar13 + 0x456) = param_1[3];
            *(ushort *)(iVar13 + 0x458) = param_1[4];
            func_0x0001092c();
            return 0;
          }
          if (0x33 < iVar5) {
            if (iVar5 == 0x3d) {
              *(char *)(iVar13 + 0x33) = (char)param_1[2];
              return 0;
            }
            if (iVar5 != 0x5f) {
              return 2;
            }
            uVar2 = param_1[2];
            *(char *)(DAT_fff004c4 + 0x1b) = (char)uVar2;
            if ((char)uVar2 != '\0') {
              *(undefined1 *)(iVar12 + 0x1b) = 1;
              return 0;
            }
            return 0;
          }
          if (iVar5 == 0x22) {
            uVar10 = 0;
            if ((*pcVar14 != '\x04') && (*pcVar14 != '\x06')) {
              return 0;
            }
            *(char *)(iVar6 + 0x3a0) = (char)param_1[2];
            cVar1 = *(char *)((int)param_1 + 5);
            *(char *)(iVar6 + 0x3a1) = cVar1;
            iVar5 = DAT_fff00908;
            iVar13 = DAT_fff00904;
            if (*(char *)(iVar6 + 0x3a0) == '\0') {
              return 0;
            }
            if (cVar1 != '\0') {
              iVar12 = DAT_fff00904 + DAT_fff00908;
              for (; uVar10 < *(ushort *)(iVar12 + 0x14); uVar10 = uVar10 + 1 & 0xff) {
                iVar7 = uVar10 * 0xc + iVar13 + iVar5;
                *(char *)(iVar7 + 0x1b) = *(char *)(iVar6 + 0x3a0) + *(char *)(iVar6 + 0x3a1);
                *(byte *)(iVar7 + 0x1c) = *(byte *)(iVar7 + 0x1c) & 0xfe;
              }
              func_0x0000f2aa(DAT_fff00910,DAT_fff0090c);
              return 0;
            }
            return 0;
          }
          if (iVar5 == 0x26) {
            *DAT_fff004dc = *(undefined4 *)(param_1 + 2);
            puVar4[1] = *(undefined4 *)(param_1 + 4);
            uVar10 = *(uint *)(param_1 + 6);
            puVar4[2] = uVar10;
            if (0x400 < uVar10) {
              puVar4[2] = 0x400;
              return 0;
            }
            return 0;
          }
          if (iVar5 != 0x32) {
            return 2;
          }
          uVar8 = 8;
          param_1 = param_1 + 2;
          pcVar14 = DAT_fff00918;
          goto LAB_fff00646;
        }
        if (iVar5 == 0x1d) {
          uVar8 = 0x44;
          iVar13 = 0x27c;
        }
        else {
          if (iVar5 < 0x1e) {
            if (iVar5 == 0x13) {
              *(char *)(iVar6 + 0x1e7) = (char)param_1[2];
              return 0;
            }
            if (iVar5 == 0x14) {
              *DAT_fff004c8 = *(undefined4 *)(param_1 + 2);
              return 0;
            }
            if (iVar5 != 0x15) {
              return 2;
            }
            *(ushort *)(iVar13 + 0x1d0) = param_1[2] * 5;
            *(char *)(iVar13 + 0x1dc) = (char)param_1[3];
            func_0x0000f0fc(iVar13 + 0x1d4,param_1 + 4,4);
            param_1 = param_1 + 6;
            uVar8 = 4;
            pcVar14 = (char *)(iVar13 + 0x1d8);
            goto LAB_fff00646;
          }
          if (iVar5 == 0x1e) {
            uVar8 = 0x44;
            iVar13 = 0x2c0;
          }
          else {
            if (iVar5 != 0x1f) {
              if (iVar5 != 0x20) {
                return 2;
              }
              func_0x0000f0fc(DAT_fff00900 + 0x44,param_1 + 2,4);
              *(char *)(local_24 + 0x16) = (char)local_2c;
              if ((*(byte *)(puVar11 + 1) & 1) == 0) {
                *(ushort *)(local_24 + 4) = *(ushort *)(local_24 + 4) & 0xfffe;
                return 0;
              }
              return 0;
            }
            uVar8 = 0x54;
            iVar13 = 0x304;
          }
        }
      }
LAB_fff007b4:
      param_1 = param_1 + 2;
      pcVar14 = pcVar14 + iVar13;
LAB_fff00646:
      func_0x0000f0fc(pcVar14,param_1,uVar8);
      return 0;
    }
    uVar15 = DAT_fff004b4 - 0x11;
    if (uVar10 == uVar15) {
      mib_set_vif_flag_1000000(param_1,iVar7,uVar10,uVar10 - uVar15);
      local_32 = param_1[3];
      local_34 = (byte)param_1[2];
      local_30 = param_1 + 4;
      local_33 = *(undefined1 *)((int)param_1 + 5);
      if (7 < local_34) {
        return 2;
      }
      if (*(ushort *)(DAT_fff004d0 + (uint)local_34 * 2) < local_32) {
        return 2;
      }
      func_0x000155f4(&local_34);
      return 0;
    }
    if ((int)uVar15 <= (int)uVar10) {
      func_0x0001b160(PTR_g_tcm_state_fff004bc,iVar7,PTR_DAT_fff004c0);
                    /* WARNING: Bad instruction - Truncating control flow here */
      halt_baddata();
    }
    if (uVar10 == 7) {
      *(undefined4 *)(iVar6 + 0x124) = *(undefined4 *)(param_1 + 2);
      return 0;
    }
    if (7 < uVar10) {
      if (uVar10 == 0xb) {
        *DAT_fff004e0 = *(undefined4 *)(param_1 + 2);
        uVar8 = *(undefined4 *)(param_1 + 4);
        puVar11 = DAT_fff004e4;
LAB_fff00340:
        *puVar11 = uVar8;
        return 0;
      }
      if (uVar10 < 0xc) {
        if (uVar10 != 8) {
          if (uVar10 != 9) {
            if (uVar10 != 10) {
              return 2;
            }
            uVar2 = param_1[2];
            *(ushort *)(iVar6 + 0x128) = uVar2;
            if (uVar2 < 0x11) {
              *(undefined2 *)(puVar3 + (uint)*(byte *)(iVar13 + 0x1a) * 2 + -4) = 1;
              return 0;
            }
            *(undefined2 *)(iVar6 + 0x128) = 0x10;
            *(undefined2 *)(puVar3 + (uint)*(byte *)(iVar13 + 0x1a) * 2 + -4) = 0;
            return 0;
          }
          puVar9 = param_1 + 4;
          uVar10 = (param_1[3] - 4 & 0x7ffff) >> 3;
          iVar13 = 0;
          if (-1 < (int)((uint)param_1[2] << 0x1b)) {
            for (; iVar13 < (int)uVar10; iVar13 = iVar13 + 1) {
              **(undefined4 **)puVar9 = *(undefined4 *)(puVar9 + 2);
              puVar9 = puVar9 + 4;
            }
            return 0;
          }
          if (uVar10 < 2) {
            return 2;
          }
          puVar11 = *(undefined4 **)puVar9;
          if (*(undefined4 **)(param_1 + 8) < puVar11) {
            return 2;
          }
          uVar8 = *(undefined4 *)(param_1 + 6);
          for (iVar13 = ((uint)((int)*(undefined4 **)(param_1 + 8) - (int)puVar11) >> 2) + 1;
              0 < iVar13; iVar13 = iVar13 + -1) {
            *puVar11 = uVar8;
            puVar11 = puVar11 + 1;
          }
          return 0;
        }
        uVar8 = *(undefined4 *)(param_1 + 2);
        puVar11 = DAT_fff004cc;
        goto LAB_fff00340;
      }
      if (uVar10 == 0x1000) {
        *(char *)(iVar13 + 0x26) = (char)param_1[2];
        func_0x0001253a(pcVar14);
        return 0;
      }
      if (uVar10 != 0x1001) {
        return 2;
      }
      uVar8 = 8;
      iVar13 = 0x358;
      goto LAB_fff007b4;
    }
    iVar13 = func_0x0001b160(PTR_g_tcm_state_fff004bc,iVar7,uVar10,uVar10);
    iVar5 = 7;
    param_1 = (ushort *)(uint)_DAT_00000043;
    uVar10 = 0xfff003b0;
  } while( true );
}



/* ======================================================================
 * fff0087a  mib_read_dispatch
 * ====================================================================== */

/* mib_read_dispatch(u16 *req) -- WSM_READ_MIB (0x0005) handler, mib.c.
   Called from wsm_h_05_read_mib at 0x00010E6E.  Returns 0 / 2 (unsupported).
   Pure compare ladder, no switch8, so the decompilation is trustworthy.
   
   Rejects if if_id > 3, or if_id > 1 for any id other than 0x100C.
   Pivots: A = 0x100C+0x17 = 0x1023, B = 0x100C+1 = 0x100D.
   
   COMPLETE set of readable MIBs (18):
     0x0000 DOT11_STATION_ID       returns 3 u16 = the MAC address
     0x0009 RW_FW_REG              arbitrary firmware memory READ (see below)
     0x100A STATISTICS_TABLE
     0x100C COUNTERS_TABLE
     0x100D BLOCK_ACK_INFO
     0x100E BLOCK_ACK_POLICY
     0x1019 P2P_PS_MODE_INFO       0x10 bytes from vif+0x1AC
     0x101F GRP_SEQ_COUNTER
     0x1022 GPIO_COMMAND
     0x1023 TSF_COUNTER            8 bytes, via func 0x0000F6A6(2)
     0x1036 AMPDUCOUNTERS_TABLE    0x28 bytes from DAT_fff00d3c
     0x1037 TXPIPE_TABLE           0x28 bytes from PTR_DAT_fff00d24-0x78
     0x1038 BACKOFF_DBG
     0x1040 REQ_PKT_STATUS         per-link scan, up to 30 entries of 0x170 bytes
     0x1042 TX_POWER_INFO          0x28 bytes
     0x1043 HW_INFO
     0x1044                        (no name in either driver header)
     0x1045                        (no name in either driver header)
   Everything else returns 2.  Note 0x1041 TPA_DEBUG_INFO is NOT readable.
   
   0x0009 RW_FW_REG read format -- req[2] = flags, req[3] = byte length:
     flags bit 4 clear : payload is a list of u32 addresses; each is
                         dereferenced and replaced by its contents,
                         n = (len-4)/4 capped at 16
     flags bit 4 set   : payload is { u32 start, ..., u32 end }; words are
                         read from start upward while p <= end, max 16
     on success flags |= 2 and req[3] is the byte count written back
   The write direction is in mib_write_dispatch: bit 4 clear means the payload
   is a list of { u32 addr, u32 value } pairs, bit 4 set means fill the range
   [start, end] with one value.  Together these are a full arbitrary
   read/write channel into firmware memory over plain WSM -- no UART needed.
   Neither driver ever issues this MIB. */

undefined4 mib_read_dispatch(ushort *param_1)

{
  ushort uVar1;
  undefined *puVar2;
  ushort uVar3;
  int iVar4;
  int iVar5;
  undefined *puVar6;
  undefined4 uVar7;
  int iVar8;
  byte bVar9;
  int *piVar10;
  uint *puVar11;
  int *piVar12;
  int iVar13;
  uint uVar14;
  char cVar15;
  uint *puVar16;
  ushort *puVar17;
  uint uVar18;
  int iVar19;
  bool bVar20;
  bool bVar21;
  undefined8 uVar22;
  int local_7c [18];
  int local_34;
  int local_30;
  int local_2c;
  byte *local_28;
  byte *local_24;
  int local_20;
  int local_1c;
  undefined *local_18;
  
  uVar14 = DAT_fff0091c;
  uVar3 = *param_1;
  local_7c[0x11] = (int)uVar3;
  uVar18 = (uint)*(byte *)(DAT_fff00904 + 10);
  if ((3 < uVar18) || ((1 < uVar18 && (local_7c[0x11] != DAT_fff0091c)))) {
    param_1[2] = uVar3;
    goto LAB_fff01090;
  }
  iVar19 = uVar18 * 0x3b0 + DAT_fff00904;
  param_1[0] = 0;
  iVar13 = DAT_fff00d40;
  iVar8 = DAT_fff00d34;
  iVar5 = DAT_fff00d30;
  puVar2 = PTR_DAT_fff00d24;
  puVar6 = PTR_DAT_fff00d20;
  param_1[1] = 0;
  uVar14 = uVar14 + 0x17;
  iVar4 = local_7c[0x11] - uVar14;
  puVar16 = (uint *)(param_1 + 4);
  if (local_7c[0x11] == uVar14) {
    uVar22 = func_0x0000f6a6(2);
    *(undefined8 *)puVar16 = uVar22;
LAB_fff00b46:
    uVar3 = 8;
  }
  else if (local_7c[0x11] < (int)uVar14) {
    uVar14 = DAT_fff0091c + 1;
    iVar5 = local_7c[0x11] - uVar14;
    if (local_7c[0x11] == uVar14) {
      *(undefined1 *)puVar16 = 0x10;
      uVar3 = 4;
      *(undefined1 *)((int)param_1 + 9) = 4;
      param_1[5] = 0;
    }
    else {
      if (local_7c[0x11] < (int)uVar14) {
        if (local_7c[0x11] == 0) {
          param_1[4] = *(ushort *)(PTR_DAT_fff00d20 + -0x10);
          param_1[5] = *(ushort *)(puVar6 + -0xe);
          param_1[6] = *(ushort *)(puVar6 + -0xc);
          goto LAB_fff00b46;
        }
        if (local_7c[0x11] == 9) {
          func_0x0001af30(local_7c,0x40);
          uVar3 = param_1[2];
          iVar5 = 0;
          uVar1 = param_1[3];
          param_1[3] = 0;
          uVar18 = (uVar1 - 4 & 0x3ffff) >> 2;
          uVar14 = 0x10;
          if (uVar18 < 0x10) {
            uVar14 = uVar18;
          }
          puVar11 = puVar16;
          if ((int)((uint)uVar3 * 0x8000000) < 0) {
            if (uVar14 < 2) {
              param_1[2] = (ushort)local_7c[0x11];
              return 2;
            }
            piVar10 = (int *)*puVar16;
            piVar12 = *(int **)(param_1 + 6);
            for (; (piVar10 <= piVar12 && (iVar5 < 0x10)); iVar5 = iVar5 + 1) {
              local_7c[iVar5] = *piVar10;
              piVar10 = piVar10 + 1;
              param_1[3] = param_1[3] + 4;
            }
          }
          else {
            for (; iVar5 < (int)uVar14; iVar5 = iVar5 + 1) {
              local_7c[iVar5] = *(int *)*puVar11;
              puVar11 = puVar11 + 1;
              param_1[3] = param_1[3] + 4;
            }
          }
          if (param_1[3] != 0) {
            func_0x0000f0fc(param_1 + 6,local_7c);
            param_1[3] = param_1[3] + 4;
            *(ushort *)puVar16 = uVar3 | 2;
            param_1[5] = param_1[3];
          }
          goto LAB_fff01082;
        }
        if (local_7c[0x11] + DAT_fff00920 != 0) {
          bVar20 = local_7c[0x11] + DAT_fff00920 == 2;
          goto LAB_fff00926;
        }
        *(ushort *)puVar16 = *(ushort *)(iVar19 + 0x380);
        *(undefined1 *)(param_1 + 5) = *(undefined1 *)(iVar19 + 0x382);
        *(undefined1 *)((int)param_1 + 0xb) = *(undefined1 *)(iVar19 + 899);
      }
      else {
        if (iVar5 != 1) {
          if (iVar5 == 0xc) {
            func_0x0000f0fc(puVar16,iVar19 + 0x1ac,0x10);
LAB_fff01048:
            uVar3 = 0x10;
          }
          else {
            if (iVar5 != 0x12) {
              bVar21 = iVar5 == 0x15;
              goto LAB_fff009a0;
            }
            iVar5 = func_0x0000139c(uVar18,puVar16);
            if (iVar5 != 0) goto LAB_fff00b46;
            uVar3 = 0;
          }
          goto LAB_fff0106c;
        }
        func_0x000020a8(uVar18,DAT_fff00d50 + -2);
        *(undefined4 *)(param_1 + 4) = *(undefined4 *)(DAT_fff00d50 + -2);
      }
LAB_fff009ae:
      uVar3 = 4;
    }
  }
  else {
    uVar18 = 0x28;
    if (iVar4 != 0x1f) {
      if (iVar4 < 0x20) {
        if (iVar4 == 0x13) {
          func_0x0000f0fc(puVar16,DAT_fff00d3c,0x28);
          puVar6 = DAT_fff00d3c;
        }
        else if (iVar4 == 0x14) {
          func_0x0000f0fc(puVar16,PTR_DAT_fff00d24 + -0x78,0x28);
          puVar6 = PTR_DAT_fff00d24 + -0x78;
        }
        else {
          if (iVar4 != 0x15) {
            bVar20 = iVar4 == 0x1d;
            if (!bVar20) goto LAB_fff00926;
            uVar14 = 0;
            local_7c[0xd] = 0;
            local_7c[0xe] = 0;
            local_7c[0xb] = *(int *)(param_1 + 2);
            local_7c[10] = DAT_fff00d2c;
            local_7c[9] = DAT_fff00d30;
            local_1c = DAT_fff00d34 + 0x160;
            local_20 = DAT_fff00d34 + 0x100;
            uVar18 = 0;
            if (local_7c[0xb] != 0xffffffff) {
              *puVar16 = local_7c[0xb];
              *(undefined1 *)(param_1 + 6) = 0xff;
              *(byte *)((int)param_1 + 0xd) =
                   *(byte *)(iVar8 + 0xa4) | *(byte *)(iVar8 + 0x110) |
                   *(byte *)(iVar8 + 0x17c) | *(byte *)(iVar8 + 0x1e8);
              param_1[3] = 6;
              while (*(uint *)(local_7c[10] + 8) != local_7c[0xb]) {
                uVar14 = uVar14 + 1;
                local_7c[10] = local_7c[10] + 0x170;
                if (0x1d < uVar14) goto LAB_fff01082;
              }
              local_24 = (byte *)(local_7c[10] + 0x60);
              bVar9 = *(byte *)(DAT_fff00d54 + (uint)*local_24);
              *(undefined1 *)(param_1 + 6) = 0;
              param_1[7] = *(short *)(local_7c[10] + 0x72) << 8 | *(ushort *)(local_7c[10] + 0x70);
              *(undefined4 *)(param_1 + 8) = *(undefined4 *)(local_7c[10] + 0x58);
              *(undefined4 *)(param_1 + 10) = *(undefined4 *)(local_7c[10] + 0x80);
              *(undefined4 *)(param_1 + 0xc) = *(undefined4 *)(DAT_fff00d58 + 0x18);
              *(undefined4 *)(param_1 + 0xe) = *(undefined4 *)(DAT_fff00d5c + 8);
              iVar4 = DAT_fff00d60;
              *(undefined4 *)(param_1 + 0x10) = *(undefined4 *)(DAT_fff00d60 + 0x38);
              iVar13 = DAT_fff00d40;
              *(uint *)(param_1 + 0x12) =
                   (uint)*(byte *)(DAT_fff00d58 + -0x15) |
                   (uint)*(byte *)(DAT_fff00d5c + -0x25) << 8 | (uint)*(byte *)(iVar4 + 0xb) << 0x10
              ;
              *(undefined1 *)(param_1 + 0x14) = *(undefined1 *)(iVar13 + 4);
              *(undefined1 *)((int)param_1 + 0x29) = *(undefined1 *)(iVar13 + 5);
              *(char *)(param_1 + 0x15) = (char)*(undefined2 *)(iVar13 + 10);
              bVar9 = *DAT_fff00d64 & 3 | (bVar9 & 3) << 2;
              *(byte *)((int)param_1 + 0x2b) = bVar9;
              bVar9 = bVar9 | (*(byte *)(iVar8 + 0xa3) & 1) << 4;
              *(byte *)((int)param_1 + 0x2b) = bVar9;
              bVar9 = bVar9 | (*(byte *)(iVar8 + 0x10f) & 1) << 5;
              *(byte *)((int)param_1 + 0x2b) = bVar9;
              bVar9 = bVar9 | (*(byte *)(iVar8 + 0x17b) & 1) << 6;
              *(byte *)((int)param_1 + 0x2b) = bVar9;
              *(byte *)((int)param_1 + 0x2b) = bVar9 | *(char *)(iVar8 + 0x1e7) << 7;
              iVar13 = DAT_fff00d68;
              uVar14 = 0;
              do {
                iVar4 = uVar14 * 0x38 + iVar5;
                if (*(char *)(iVar4 + 0x2f) == '\0') {
                  local_7c[0xe] =
                       local_7c[0xe] | (*(byte *)(iVar4 + 0x10) & 0xf) << ((uVar14 & 0x3f) << 2);
                }
                else {
                  local_7c[0xd] =
                       local_7c[0xd] | (*(byte *)(iVar4 + 0x10) & 0xf) << ((uVar14 & 0x3f) << 2);
                }
                uVar14 = uVar14 + 1;
              } while ((int)uVar14 < 8);
              *(int *)(param_1 + 0x16) = local_7c[0xe];
              *(int *)(param_1 + 0x18) = local_7c[0xd];
              *(undefined1 *)(param_1 + 0x1c) = *(undefined1 *)(iVar13 + 0xc);
              *(undefined1 *)((int)param_1 + 0x39) = *(undefined1 *)(DAT_fff00d6c + 0x15);
              iVar5 = DAT_fff00d70;
              *(undefined1 *)(param_1 + 0x1d) = *(undefined1 *)(DAT_fff00d70 + 0x14);
              *(char *)((int)param_1 + 0x3b) = (char)*(undefined2 *)(iVar5 + 0x18);
              *(undefined1 *)(param_1 + 0x1e) = *(undefined1 *)(iVar8 + 0xa4);
              *(undefined1 *)((int)param_1 + 0x3d) = *(undefined1 *)(iVar8 + 0x110);
              *(undefined1 *)(param_1 + 0x1f) = *(undefined1 *)(iVar8 + 0x17c);
              *(undefined1 *)((int)param_1 + 0x3f) = *(undefined1 *)(iVar8 + 0x1e8);
              param_1[0x20] = *(ushort *)(iVar8 + 0xa6);
              param_1[0x21] = *(ushort *)(iVar8 + 0x112);
              param_1[0x22] = *(ushort *)(iVar8 + 0x17e);
              param_1[0x23] = *(ushort *)(iVar8 + 0x1ea);
              *(undefined4 *)(param_1 + 0x24) = *(undefined4 *)(local_7c[10] + 0x40);
              uVar7 = func_0x0000e6b8();
              *(undefined4 *)(param_1 + 0x26) = uVar7;
              *(undefined4 *)(param_1 + 0x28) = *(undefined4 *)(local_7c[10] + 0x6c);
              *(undefined4 *)(param_1 + 0x2a) = *(undefined4 *)(local_7c[10] + 0x68);
              param_1[3] = 0x50;
              if (*(short *)(local_24 + 0x10) == 0xff) {
                uVar18 = 0x40;
              }
              iVar5 = 0;
              do {
                iVar8 = 0;
                cVar15 = '\0';
                do {
                  if (uVar18 != 0) break;
                  iVar13 = iVar5 * 0x6c + DAT_fff011a4 + iVar8 * 0x18;
                  if ((*(char *)(iVar13 + 0xac) == '\x01') || (*(char *)(iVar13 + 0xac) == '\0')) {
                    cVar15 = '\0';
                    for (iVar4 = *(int *)(iVar13 + 0xb8); iVar4 != 0; iVar4 = *(int *)(iVar4 + 0x3c)
                        ) {
                      if (iVar4 == local_7c[10] + 0x54) {
                        uVar18 = 0x20;
                        *(byte *)((int)param_1 + 0x35) = (byte)(iVar8 << 4) | (byte)iVar5;
                        *(undefined1 *)(param_1 + 0x1b) = *(undefined1 *)(iVar13 + 0xac);
                        *(undefined1 *)((int)param_1 + 0x37) = *(undefined1 *)(iVar13 + 0xaf);
                        break;
                      }
                      cVar15 = cVar15 + '\x01';
                    }
                  }
                  iVar8 = iVar8 + 1;
                } while (iVar8 < 4);
                if ((int)(uVar18 << 0x1a) < 0) {
                  func_0x0000ad76();
                  iVar5 = func_0x0000038c(1,DAT_fff011a8,0);
                  if (iVar5 == 2) {
                    func_0x000002f4();
                  }
                  *(char *)(param_1 + 0x1c) = cVar15;
                  break;
                }
                iVar5 = iVar5 + 1;
              } while (iVar5 < 4);
              local_28 = (byte *)(local_7c[10] + 0xc0);
              iVar5 = 0;
              local_2c = local_7c[10] + 0x54;
              do {
                if (*(int *)((uint)*local_28 * 0x40 + DAT_fff011a4 + iVar5 * 4 + 0x490) == local_2c)
                {
                  if (uVar18 == 0) {
                    *(byte *)((int)param_1 + 0x35) = *local_28;
                    *(char *)(param_1 + 0x1b) = (char)iVar5;
                    func_0x0000b74c(local_7c[10],0x18);
                  }
                  uVar18 = uVar18 | 0x10;
                  *(undefined4 *)((uint)*local_28 * 0x40 + DAT_fff011a4 + iVar5 * 4 + 0x490) = 0;
                  break;
                }
                iVar5 = iVar5 + 1;
              } while (iVar5 < 0x10);
              iVar5 = DAT_fff011ac;
              iVar8 = 0;
              do {
                if (*(int *)(iVar8 * 4 + DAT_fff011ac + 0x10) == local_2c) {
                  if (uVar18 == 0) {
                    *(char *)((int)param_1 + 0x35) = (char)*(undefined4 *)(DAT_fff011ac + 8);
                    *(char *)(param_1 + 0x1b) = (char)*(undefined4 *)(iVar5 + 0xc);
                    *(char *)((int)param_1 + 0x37) = (char)iVar8;
                    func_0x0000b74c(local_7c[10],0x18);
                  }
                  uVar18 = uVar18 | 8;
                  *(undefined4 *)(iVar8 * 4 + DAT_fff011ac + 0x10) = 0;
                  break;
                }
                iVar8 = iVar8 + 1;
              } while (iVar8 < 0x40);
              local_7c[0xf] = 0;
              iVar5 = *DAT_fff011b0;
              local_7c[0xc] = *DAT_fff011b0;
              while (iVar8 = iVar5, iVar8 != 0) {
                local_7c[0xf] = local_7c[0xf] + 1;
                if (iVar8 == local_7c[10]) {
                  if (uVar18 == 0) {
                    func_0x0000b74c(local_7c[10],0x18);
                    *(char *)((int)param_1 + 0x35) = (char)local_7c[0xf];
                  }
                  uVar18 = uVar18 | 4;
                  *(undefined4 *)(local_7c[0xc] + 4) = *(undefined4 *)(iVar8 + 4);
                  piVar10 = DAT_fff011b0;
                  if (*DAT_fff011b0 == iVar8) {
                    *DAT_fff011b0 = *(int *)(iVar8 + 4);
                  }
                  if (piVar10[1] == iVar8) {
                    piVar10[1] = local_7c[0xc];
                  }
                  *(undefined4 *)(iVar8 + 4) = 0;
                  break;
                }
                local_7c[0xc] = iVar8;
                iVar5 = *(int *)(iVar8 + 4);
              }
              uVar14 = 0;
              while (*(int *)(uVar14 * 4 + DAT_fff011a4 + 0x7f8) != local_2c) {
                uVar14 = uVar14 + 1;
                if (0x3f < (int)uVar14) goto LAB_fff00fa2;
              }
              local_30 = DAT_fff011b4;
              if (uVar18 == 0) {
                func_0x0000b74c(local_7c[10],0x18);
                *(char *)((int)param_1 + 0x35) = (char)uVar14;
                *(char *)(param_1 + 0x1b) = (char)*(undefined4 *)(local_30 + 0x30);
                *(char *)((int)param_1 + 0x37) = (char)*(undefined4 *)(local_30 + 0x34);
              }
              local_7c[0xf] = uVar14 - 1 & 0x3f;
              *(undefined4 *)(uVar14 * 4 + DAT_fff011a4 + 0x7f8) = 0;
              while (uVar14 = uVar14 + 1 & 0x3f, *(uint *)(local_30 + 0x34) != uVar14) {
                iVar5 = uVar14 * 4 + DAT_fff011a4;
                local_34 = iVar5 + 0x7c0;
                iVar5 = *(int *)(iVar5 + 0x7f8);
                if (iVar5 != 0) {
                  func_0x0000b74c(iVar5 + -0x54,0x18);
                  *(undefined4 *)(local_34 + 0x38) = 0;
                }
              }
              *(int *)(local_30 + 0x34) = local_7c[0xf];
              uVar18 = uVar18 | 2;
LAB_fff00fa2:
              iVar5 = DAT_fff011b8;
              iVar8 = 0;
              do {
                if (*(int *)(iVar8 * 4 + DAT_fff011b8 + 0x14) == local_2c) {
                  if (uVar18 == 0) {
                    *(char *)((int)param_1 + 0x35) = (char)iVar8;
                    *(char *)(param_1 + 0x1b) = (char)*(undefined4 *)(iVar5 + 0xc);
                    *(char *)((int)param_1 + 0x37) = (char)*(undefined4 *)(iVar5 + 0x10);
                  }
                  func_0x0000d254();
                  uVar18 = uVar18 | 1;
                  break;
                }
                iVar8 = iVar8 + 1;
              } while (iVar8 < 0x40);
              if (uVar18 == 0) {
                func_0x0000b74c(local_7c[10],0x18);
              }
              func_0x0000f034(DAT_fff011bc,0x200000);
              func_0x0000f034(DAT_fff011bc,0x100000);
              *(char *)(param_1 + 0x1a) = (char)uVar18;
              goto LAB_fff01082;
            }
            *(undefined1 *)puVar16 = *(undefined1 *)(DAT_fff00d34 + 0xa4);
            *(undefined1 *)((int)param_1 + 9) = *(undefined1 *)(iVar8 + 0x110);
            *(undefined1 *)(param_1 + 5) = *(undefined1 *)(iVar8 + 0x17c);
            *(undefined1 *)((int)param_1 + 0xb) = *(undefined1 *)(iVar8 + 0x1e8);
            *(undefined1 *)(param_1 + 6) = *(undefined1 *)(iVar8 + 0xa5);
            *(undefined1 *)((int)param_1 + 0xd) = *(undefined1 *)(iVar8 + 0x111);
            *(undefined1 *)(param_1 + 7) = *(undefined1 *)(iVar8 + 0x17d);
            *(undefined1 *)((int)param_1 + 0xf) = *(undefined1 *)(iVar8 + 0x1e9);
            param_1[8] = *(ushort *)(iVar8 + 0xa6);
            param_1[9] = *(ushort *)(iVar8 + 0x112);
            param_1[10] = *(ushort *)(iVar8 + 0x17e);
            param_1[0xb] = *(ushort *)(iVar8 + 0x1ea);
            goto LAB_fff01048;
          }
          func_0x0000f0fc(puVar16,PTR_DAT_fff00d24 + -0x28,0x28);
          puVar6 = PTR_DAT_fff00d24 + -0x28;
        }
        func_0x0000f12e(puVar6,0x28);
      }
      else {
        if (iVar4 == 0x20) {
          func_0x0000f0fc(puVar16,DAT_fff011bc + 0x20,0x20);
          param_1[3] = 0x20;
          goto LAB_fff01082;
        }
        if (iVar4 != 0x21) {
          bVar21 = false;
          if (iVar4 == 0x22) {
            func_0x0000f0fc(puVar16,DAT_fff00d38,8);
            goto LAB_fff01082;
          }
LAB_fff009a0:
          bVar20 = false;
          if (bVar21) {
            *(undefined1 *)puVar16 = 1;
            iVar5 = DAT_fff00d28;
            *(undefined1 *)((int)param_1 + 9) = 0xff;
            param_1[5] = (ushort)*(undefined4 *)(iVar5 + 4);
            goto LAB_fff009ae;
          }
LAB_fff00926:
          if (!bVar20) {
            param_1[2] = uVar3;
LAB_fff01090:
            param_1[3] = 0;
            return 2;
          }
          local_18 = PTR_DAT_fff00d20;
          func_0x0000f0fc(puVar16,PTR_DAT_fff00d20,0x20);
          func_0x0000f12e(local_18,0x20);
          puVar17 = param_1 + 0x14;
          local_7c[0x10] = 0x20;
          if (uVar18 < 3) {
            func_0x0000f0fc(puVar17,iVar19 + 0x68,0x2c);
            func_0x0000f12e(iVar19 + 0x68,0x2c);
            func_0x0000f0fc(param_1 + 0x2a,iVar19 + 0x94,0x10);
            func_0x0000f12e(iVar19 + 0x94,0x10);
            puVar17 = param_1 + 0x32;
            local_7c[0x10] = 0x5c;
          }
          func_0x0000f0fc(puVar17,PTR_DAT_fff00d24,0x24);
          func_0x0000f12e(PTR_DAT_fff00d24,0x24);
          uVar3 = (short)local_7c[0x10] + 0x24;
          goto LAB_fff0106c;
        }
        *(uint *)(PTR_DAT_fff00d24 + -0x50) = (uint)*(ushort *)(DAT_fff00d40 + 10);
        *(uint *)(puVar2 + -0x4c) = *(int *)(DAT_fff00d44 + 0xc) - *(int *)(DAT_fff00d44 + 8) & 0x3f
        ;
        *(uint *)(puVar2 + -0x48) = *(int *)(iVar13 + 0x10) - *(int *)(iVar13 + 0xc) & 0x3f;
        puVar6 = PTR_DAT_fff00d20;
        *(undefined4 *)(puVar2 + -0x44) = *(undefined4 *)(PTR_DAT_fff00d20 + 0x24);
        *(undefined4 *)(puVar2 + -0x40) = *(undefined4 *)(puVar6 + 0x28);
        *(undefined4 *)(puVar2 + -0x3c) = *(undefined4 *)(puVar6 + 0x2c);
        *(undefined4 *)(puVar2 + -0x38) = *DAT_fff00d48;
        *(undefined4 *)(puVar2 + -0x34) = *DAT_fff00d4c;
        func_0x0000f12e(PTR_DAT_fff00d20 + 0x20,0x10);
        func_0x0000f0fc(puVar16,PTR_DAT_fff00d24 + -0x50,0x28);
      }
      param_1[3] = 0x28;
      goto LAB_fff01082;
    }
    param_1[3] = 4;
    *(undefined1 *)((int)param_1 + 9) = 0xff;
    if ((char)param_1[2] != '\x01') goto LAB_fff01082;
    *(undefined1 *)((int)param_1 + 9) = 0;
    func_0x0000f0fc(param_1 + 6,DAT_fff011c0,0x100);
    uVar3 = param_1[3] + 0x100;
  }
LAB_fff0106c:
  param_1[3] = uVar3;
LAB_fff01082:
  param_1[2] = (ushort)local_7c[0x11];
  return 0;
}



/* ======================================================================
 * fff01094  fw_global_state_init
 * ====================================================================== */

void fw_global_state_init(void)

{
  undefined *puVar1;
  undefined *puVar2;
  undefined4 *puVar3;
  undefined2 *puVar4;
  undefined4 uVar5;
  int iVar6;
  undefined4 uVar7;
  uint uVar8;
  uint uVar9;
  int iVar10;
  
  puVar1 = PTR_DAT_fff011c4;
  *(undefined4 *)(PTR_DAT_fff011c4 + 0x30) = 0;
  func_0x0000f034(DAT_fff011bc + 4,0x80000000);
  uVar9 = 0;
  PTR_g_tcm_state_fff011c8[6] = 2;
  do {
    iVar6 = uVar9 * 0x3b0 + DAT_fff011cc;
    *(undefined4 *)(iVar6 + 0x11c) = 0x200;
    *(undefined4 *)(iVar6 + 0x120) = 0x200;
    *(undefined4 *)(iVar6 + 0x124) = 300;
    *DAT_fff011d0 = 1;
    *(undefined2 *)(iVar6 + 0x128) = 8;
    *(undefined1 *)(iVar6 + 0x33) = 0;
    *(undefined1 *)(iVar6 + 0x1e5) = 0;
    *(undefined1 *)(iVar6 + 0x1e6) = 0;
    *(undefined1 *)(iVar6 + 0x111) = 0xff;
    *(undefined1 *)(iVar6 + 0x112) = 4;
    *(undefined1 *)(iVar6 + 300) = 0xff;
    iVar10 = DAT_fff011d4;
    *(undefined1 *)(iVar6 + 0x12d) = 0xff;
    *(short *)(iVar6 + 0x1e0) = (short)iVar10;
    *(undefined1 *)(iVar6 + 0x26) = 0;
    *(undefined4 *)(iVar6 + iVar10 + 0xb6) = 2;
    *(undefined2 *)(iVar6 + 0x378) = 1;
    *(undefined2 *)(iVar6 + 0x37a) = 0;
    *(undefined2 *)(iVar6 + 0x37e) = 0;
    *(undefined2 *)(iVar6 + 0x380) = 0;
    *(undefined1 *)(iVar6 + 0x382) = 0;
    *(undefined1 *)(iVar6 + 899) = 0;
    *(undefined2 *)(iVar6 + 0x130) = 0;
    *(undefined4 *)(iVar6 + 0x138) = 0;
    *(undefined1 *)(iVar6 + 0x218) = 0;
    *(undefined1 *)(iVar6 + 0x22c) = 0;
    *(undefined1 *)(iVar6 + 0x240) = 0;
    *(undefined1 *)(iVar6 + 0x1e7) = 0;
    *(undefined2 *)(iVar6 + 0x1d0) = 0;
    *(undefined1 *)(iVar6 + 0x1dc) = 8;
    func_0x0000f12e(iVar6 + 0x1d4,4);
    func_0x0000f12e(iVar6 + 0x1d8,4);
    func_0x0000f12e(iVar6 + 0x370,8);
    func_0x0000f12e(iVar6 + 0x3a8,0x14);
    func_0x0000f12e(iVar6 + 0x68,0x2c);
    uVar9 = uVar9 + 1;
  } while (uVar9 < 2);
  uVar9 = 0;
  do {
    iVar10 = uVar9 * 0x98 + DAT_fff011d8;
    *(undefined1 *)(iVar10 + 0x490) = 7;
    func_0x000136a6(uVar9 & 0xff,DAT_fff011dc);
    *(undefined4 *)(iVar10 + 0x500) = 0;
    if (uVar9 < 2) {
      uVar7 = 9;
    }
    else {
      uVar7 = 0x14;
    }
    uVar9 = uVar9 + 1;
    *(undefined4 *)(iVar10 + 0x4f8) = uVar7;
    puVar2 = PTR_g_tcm_state_fff01340;
  } while (uVar9 < 3);
  *(undefined4 *)(PTR_g_tcm_state_fff01340 + 0x44) = 300;
  uVar9 = 0;
  do {
    *(undefined4 *)(puVar2 + uVar9 * 0x188 + 0x48) = 0;
    uVar8 = uVar9 + 1;
    *(undefined4 *)(puVar2 + uVar9 * 0xc + 0x358) = 0;
    *(undefined4 *)(puVar2 + uVar9 * 0xc + 0x35c) = 0;
    *(undefined4 *)(puVar2 + uVar9 * 0xc + 0x360) = 0;
    uVar9 = uVar8;
  } while (uVar8 < 2);
  *(undefined4 *)(puVar1 + 0x34) = 0;
  puVar1 = PTR_DAT_fff01344;
  *(undefined2 *)(PTR_DAT_fff01344 + 0x10) = 0xff00;
  *(undefined2 *)(puVar1 + 0x12) = 0xff;
  *(undefined4 *)(puVar1 + 0x14) = DAT_fff01348;
  *DAT_fff0134c = 0;
  puVar3 = DAT_fff01350;
  *DAT_fff01350 = 0;
  puVar3[1] = 0;
  puVar3[3] = 0;
  puVar3[2] = 0;
  uVar9 = 0;
  do {
    puVar2[uVar9 * 0xa4 + 0x398] = 0;
    uVar8 = 0;
    do {
      iVar10 = uVar8 * 4;
      uVar8 = uVar8 + 1 & 0xff;
      *(undefined4 *)(puVar2 + iVar10 + uVar9 * 0xa4 + 0x3f8) = 0xffffffff;
    } while (uVar8 < 4);
    uVar8 = uVar9 + 1;
    *(undefined4 *)(puVar2 + uVar9 * 0xa4 + 0x3f4) = 0xffffffff;
    uVar7 = DAT_fff01354;
    uVar9 = uVar8;
  } while (uVar8 < 0x18);
  uVar9 = 0;
  do {
    uVar5 = DAT_fff0135c;
    puVar4 = DAT_fff01358;
    uVar8 = uVar9 + 1;
    *(undefined1 *)(DAT_fff01358 + uVar9 * 0x1d8 + 0xd6) = 0x3f;
    *(undefined1 *)((int)puVar4 + uVar9 * 0x3b0 + 0x1ad) = 0;
    *(undefined4 *)(puVar4 + uVar9 * 0x1d8 + 0xd8) = uVar5;
    *(undefined4 *)(puVar4 + uVar9 * 0x1d8 + 0xdc) = uVar7;
    *(undefined4 *)(puVar4 + uVar9 * 0x1d8 + 0xda) = uVar5;
    iVar10 = DAT_fff01360;
    uVar9 = uVar8;
  } while (uVar8 < 2);
  *(undefined2 *)(DAT_fff01360 + 0x18) = 1;
  *(undefined1 *)(iVar10 + 0x1a) = 0;
  *(undefined1 *)(iVar10 + 0x1b) = 0x50;
  *(undefined1 *)(iVar10 + 0x1c) = 0xf2;
  *(undefined1 *)(iVar10 + 0x1d) = 4;
  *(undefined2 *)(iVar10 + 0x1e) = 1;
  func_0x0000f0fc(iVar10 + 0x24,s_XRadio_P2P_fff01364,0xb);
  iVar10 = DAT_fff01360;
  *(undefined1 *)(DAT_fff01360 + 0x23) = 0xb;
  *(undefined1 *)(iVar10 + 0x47) = 2;
  *(undefined2 *)(iVar10 + 0x48) = 0xb;
  *(undefined1 *)(iVar10 + 0x4a) = 0;
  *(undefined1 *)(iVar10 + 0x4b) = 0x50;
  *(undefined1 *)(iVar10 + 0x4c) = 0xf2;
  *(undefined1 *)(iVar10 + 0x4d) = 4;
  *(undefined2 *)(iVar10 + 0x4e) = 6;
  *(undefined2 *)(iVar10 + 0x50) = 0xb;
  *(undefined1 *)(iVar10 + 0x52) = 0;
  *(undefined1 *)(iVar10 + 0x53) = 0x50;
  *(undefined1 *)(iVar10 + 0x54) = 0xf2;
  *(undefined1 *)(iVar10 + 0x55) = 4;
  *(undefined2 *)(iVar10 + 0x56) = 5;
  iVar10 = DAT_fff01370;
  *(undefined2 *)(DAT_fff01370 + 0xe) = 0xffff;
  puVar4 = DAT_fff01358;
  *(undefined4 *)(iVar10 + 0x10) = DAT_fff01348;
  puVar4[6] = 0x10;
  *puVar4 = 0x13;
  func_0x0000f730(0x13,0x14);
  iVar10 = DAT_fff01374;
  uVar9 = 0;
  do {
    iVar6 = iVar10 + uVar9 * 8;
    uVar9 = uVar9 + 1;
    *(undefined1 *)((int)puVar4 + iVar6 + 0x12) = 0;
    *(undefined1 *)((int)puVar4 + iVar6 + 0x13) = 0;
  } while (uVar9 < 8);
  func_0x00000848();
  iVar10 = DAT_fff01370;
  uVar7 = DAT_fff0135c;
  *(undefined2 *)(DAT_fff01370 + -0x74) = 0;
  *(undefined2 *)(iVar10 + -0x72) = 0;
  *(undefined2 *)(iVar10 + -0x70) = 0;
  *(undefined2 *)(iVar10 + -0x6e) = 0;
  *DAT_fff01378 = uVar7;
  func_0x00011ae6(0);
  func_0x00011ae6(1);
  return;
}



/* ======================================================================
 * fff0137c  uart_tx_drain
 * ====================================================================== */

void uart_tx_drain(void)

{
  undefined *puVar1;
  int iVar2;
  uint uVar3;
  
  puVar1 = PTR_DAT_fff014c8;
  uVar3 = *(uint *)(PTR_DAT_fff014c8 + 4);
  while ((*(uint *)(puVar1 + 8) != uVar3 &&
         (iVar2 = uart_hw_putc(puVar1[(uVar3 & 0xff) + 0xc]), iVar2 != 0))) {
    uVar3 = uVar3 + 1;
  }
  *(uint *)(puVar1 + 4) = uVar3;
  return;
}



/* ======================================================================
 * fff013a0  uart_putc
 * ====================================================================== */

void uart_putc(int param_1)

{
  undefined *puVar1;
  int iVar2;
  uint uVar3;
  undefined4 uVar4;
  
  puVar1 = PTR_DAT_fff014c8;
  if (*(int *)PTR_DAT_fff014c8 == 0) {
    uVar3 = *(uint *)(PTR_DAT_fff014c8 + 8);
    if (uVar3 - *(int *)(PTR_DAT_fff014c8 + 4) < 0x100) {
      PTR_DAT_fff014c8[(uVar3 & 0xff) + 0xc] = (char)param_1;
      *(uint *)(puVar1 + 8) = uVar3 + 1;
      if (param_1 == 10) {
        uVar4 = func_0x0000efdc();
        uart_tx_drain();
        func_0x0000eff0(uVar4);
        return;
      }
    }
  }
  else {
    do {
      iVar2 = uart_hw_putc(param_1);
    } while (iVar2 == 0);
  }
  return;
}



/* ======================================================================
 * fff013e4  uart_set_blocking
 * ====================================================================== */

void uart_set_blocking(int param_1)

{
  undefined *puVar1;
  undefined4 uVar2;
  
  puVar1 = PTR_DAT_fff014c8;
  if (param_1 != 0) {
    while (*(int *)(puVar1 + 4) != *(int *)(puVar1 + 8)) {
      uVar2 = func_0x0000efdc();
      uart_tx_drain();
      func_0x0000eff0(uVar2);
    }
  }
  *(int *)puVar1 = param_1;
  return;
}



/* ======================================================================
 * fff0141a  uart_rx_poll
 * ====================================================================== */

void uart_rx_poll(void)

{
  undefined *puVar1;
  undefined *puVar2;
  int iVar3;
  uint uVar4;
  uint uVar5;
  uint in_r3;
  uint uVar6;
  uint local_18;
  
  puVar2 = PTR_DAT_fff014cc;
  uVar6 = *(uint *)(PTR_DAT_fff014cc + 0xc);
  local_18 = in_r3;
LAB_fff01496:
  do {
    while( true ) {
      if ((0x1ff < uVar6 - *(int *)(puVar2 + 0x10)) || (iVar3 = uart_hw_getc(&local_18), iVar3 == 0)
         ) {
        return;
      }
      uart_hw_putc(local_18 & 0xff);
      puVar1 = PTR_DAT_fff014c8;
      uVar4 = local_18 & 0xff;
      if (uVar4 == 10) break;
      if ((uVar4 == 0x20) || (uVar4 == 9)) goto LAB_fff01450;
      if (uVar4 != 0x23) goto LAB_fff01464;
      puVar2[0x14] = (undefined1)local_18;
    }
  } while (puVar2[0x14] == '\r');
  goto LAB_fff01464;
LAB_fff01450:
  if ((puVar2[0x14] != ' ') && (puVar2[0x14] != '\t')) {
LAB_fff01464:
    if (((puVar2[0x14] != '#') || (uVar4 == 0xd)) || (uVar4 == 10)) {
      uVar5 = uVar6 & 0x1ff;
      puVar2[0x14] = (undefined1)local_18;
      uVar6 = uVar6 + 1;
      puVar1[uVar5 + 0x118] = (undefined1)local_18;
      *(uint *)(puVar2 + 0xc) = uVar6;
      if ((uVar4 == 0xd) || (uVar4 == 10)) {
        func_0x0000f034(DAT_fff014d0,8);
      }
    }
  }
  goto LAB_fff01496;
}



/* ======================================================================
 * fff014a0  uart_getc
 * ====================================================================== */

undefined4 uart_getc(undefined1 *param_1)

{
  undefined *puVar1;
  uint uVar2;
  
  puVar1 = PTR_DAT_fff014cc;
  uVar2 = *(uint *)(PTR_DAT_fff014cc + 0x10);
  if (*(uint *)(PTR_DAT_fff014cc + 0xc) == uVar2) {
    return 0;
  }
  *param_1 = PTR_DAT_fff014c8[(uVar2 & 0x1ff) + 0x118];
  *(uint *)(puVar1 + 0x10) = uVar2 + 1;
  return 1;
}



/* ======================================================================
 * fff014d4  uart_putc_crlf
 * ====================================================================== */

void uart_putc_crlf(int param_1)

{
  if (param_1 == 10) {
    uart_putc(0xd);
  }
  uart_putc(param_1);
  return;
}



/* ======================================================================
 * fff014ea  dbg_printf
 * ====================================================================== */

void dbg_printf(byte *param_1,uint param_2,undefined4 param_3,undefined4 param_4)

{
  char cVar1;
  byte *pbVar2;
  byte bVar3;
  int iVar4;
  char extraout_r1;
  char extraout_r1_00;
  uint uVar5;
  uint uVar6;
  int iVar7;
  uint *puVar8;
  char *pcVar9;
  int iVar10;
  bool bVar11;
  char acStack_46 [22];
  uint local_30;
  uint local_2c;
  uint local_28;
  byte *local_10;
  uint local_c [3];
  
  local_c[2] = param_4;
  local_c[1] = param_3;
  local_c[0] = param_2;
  puVar8 = local_c;
  local_10 = param_1;
  do {
    while( true ) {
      bVar3 = *local_10;
      if (bVar3 == 0) {
        return;
      }
      if (bVar3 == 0x25) break;
LAB_fff01696:
      local_10 = local_10 + 1;
      uart_putc_crlf(bVar3);
    }
    local_30 = 0x20;
    local_2c = 0;
    iVar10 = 0;
    iVar4 = -1;
    bVar11 = local_10[1] == 0x2d;
    pbVar2 = local_10 + 1;
    if (bVar11) {
      local_2c = 1;
      pbVar2 = local_10 + 2;
    }
    local_10 = pbVar2;
    local_2c = (uint)bVar11;
    uVar5 = (uint)*local_10;
    if (uVar5 - 0x30 < 10) {
      if (uVar5 != 0x30) goto LAB_fff01534;
      local_30 = uVar5;
      while (local_10 = local_10 + 1, *local_10 - 0x30 < 10) {
LAB_fff01534:
        iVar10 = (*local_10 & 0xf) + iVar10 * 10;
      }
    }
    if (*local_10 == 0x2e) {
      iVar4 = 0;
      while( true ) {
        local_10 = local_10 + 1;
        if (9 < *local_10 - 0x30) break;
        iVar4 = (*local_10 & 0xf) + iVar4 * 10;
      }
    }
    bVar3 = *local_10;
    if (((bVar3 == 0x6c) || (bVar3 == 0x4c)) || (bVar3 == 0x68)) {
      local_10 = local_10 + 1;
    }
    bVar3 = *local_10;
    if (bVar3 == 100) {
      uVar5 = *puVar8;
      bVar11 = (int)uVar5 < 0;
      if (bVar11) {
        uVar5 = -uVar5;
      }
      iVar7 = 0xb;
      do {
        iVar4 = iVar7;
        uVar5 = func_0x0001b18c(uVar5,10);
        iVar7 = iVar4 + -1;
        acStack_46[iVar4 + 1] = extraout_r1 + '0';
      } while (uVar5 != 0);
      if (bVar11) {
        iVar7 = iVar4 + -2;
        acStack_46[iVar4] = '-';
      }
      pcVar9 = acStack_46 + iVar7 + 2;
LAB_fff0160c:
      iVar7 = 0xb - iVar7;
    }
    else if (bVar3 < 0x65) {
      if (bVar3 == 0x58) {
LAB_fff015bc:
        uVar5 = *puVar8;
        iVar4 = 8;
        do {
          iVar7 = iVar4;
          uVar6 = uVar5 & 0xf;
          uVar5 = uVar5 >> 4;
          acStack_46[iVar7 + 1] = *(char *)(DAT_fff01970 + uVar6);
          iVar4 = iVar7 + -1;
        } while (uVar5 != 0);
        pcVar9 = acStack_46 + iVar7 + 1;
        iVar7 = 8 - (iVar7 + -1);
      }
      else if (bVar3 == 0x62) {
        uVar5 = *puVar8;
        for (iVar7 = 0; (iVar7 < iVar10 && (iVar7 < 0x40)); iVar7 = iVar7 + 1) {
          bVar3 = *(byte *)(uVar5 + iVar7);
          acStack_46[iVar7 * 2 + 2] = *(char *)(DAT_fff01970 + (uint)(bVar3 >> 4));
          acStack_46[iVar7 * 2 + 3] = *(char *)(DAT_fff01970 + (bVar3 & 0xf));
        }
        iVar7 = iVar7 << 1;
        pcVar9 = acStack_46 + 2;
      }
      else {
        if (bVar3 != 99) goto LAB_fff01696;
        iVar7 = 1;
        pcVar9 = acStack_46 + 2;
        acStack_46[2] = (char)*puVar8;
      }
    }
    else {
      if (bVar3 != 0x73) {
        if (bVar3 == 0x75) {
          uVar5 = *puVar8;
          iVar7 = 0xb;
          do {
            iVar4 = iVar7;
            uVar5 = func_0x0001b18c(uVar5,10);
            iVar7 = iVar4 + -1;
            acStack_46[iVar4 + 1] = extraout_r1_00 + '0';
          } while (uVar5 != 0);
          pcVar9 = acStack_46 + iVar4 + 1;
          goto LAB_fff0160c;
        }
        if (bVar3 != 0x78) goto LAB_fff01696;
        goto LAB_fff015bc;
      }
      if (iVar4 == -1) {
        iVar4 = 0xfa;
      }
      pcVar9 = (char *)*puVar8;
      if (pcVar9 == (char *)0x0) {
        pcVar9 = s_<null>_fff01974;
      }
      for (iVar7 = 0; (iVar7 < iVar4 && (pcVar9[iVar7] != '\0')); iVar7 = iVar7 + 1) {
      }
    }
    puVar8 = puVar8 + 1;
    if ((iVar10 == 0) || (iVar10 < iVar7)) {
      iVar10 = iVar7;
    }
    iVar10 = iVar10 - iVar7;
    if (local_2c == 0) {
      local_28 = local_30 & 0xff;
      while (iVar10 = iVar10 + -1, -1 < iVar10) {
        uart_putc_crlf(local_28);
      }
      while (iVar7 = iVar7 + -1, -1 < iVar7) {
        cVar1 = *pcVar9;
        pcVar9 = pcVar9 + 1;
        uart_putc_crlf(cVar1);
      }
    }
    else {
      while (iVar7 = iVar7 + -1, -1 < iVar7) {
        cVar1 = *pcVar9;
        pcVar9 = pcVar9 + 1;
        uart_putc_crlf(cVar1);
      }
      uVar5 = local_30 & 0xff;
      while (iVar10 = iVar10 + -1, -1 < iVar10) {
        uart_putc_crlf(uVar5);
      }
    }
    local_10 = local_10 + 1;
  } while( true );
}



/* ======================================================================
 * fff016f8  dbg_hexdump
 * ====================================================================== */

void dbg_hexdump(int param_1,int param_2)

{
  uint uVar1;
  
  for (uVar1 = 0; uVar1 < param_2 + 3U >> 2; uVar1 = uVar1 + 1) {
    if ((uVar1 & 3) == 0) {
      dbg_printf(s__08x___08x_fff0197c,uVar1 * 4,*(undefined4 *)(param_1 + uVar1 * 4));
    }
    else {
      dbg_printf(s__08x___08x_fff0197c + 8,*(undefined4 *)(param_1 + uVar1 * 4));
    }
  }
  uart_putc_crlf(10);
  return;
}



/* ======================================================================
 * fff0172c  dbg_puts
 * ====================================================================== */

void dbg_puts(char *param_1)

{
  char cVar1;
  
  while( true ) {
    cVar1 = *param_1;
    param_1 = param_1 + 1;
    if (cVar1 == '\0') break;
    uart_putc_crlf();
  }
  return;
}



/* ======================================================================
 * fff0198c  hif_send_debug_msg
 * ====================================================================== */

/* hif_send_debug_msg(code, data, len) -- emit a debug/trace indication to the host.
   
     msg = hif_alloc_msg_to_host(len + 8);
     if (!msg) return -1;
     *(u16 *)(msg + 2) = 0x080E;          /* MsgId, from DAT_fff019d4 */
     *(u16 *)(msg + 0) = len + 8;         /* MsgLen */
     *(u32 *)(msg + 4) = code;
     memcpy(msg + 8, data, len);
     hif_send_msg_to_host(msg);
   
   *** INDICATION 0x080E -- A LIVE FIRMWARE DEBUG CHANNEL, previously unlisted. ***
   Payload after the 4-byte header is `{ u32 code; u8 data[len]; }`.
   This project's indication inventory had 0x0800, 0x0801, 0x0805-0x080A, 0x080C,
   0x080D and 0x080F; **0x080E was missing** because this emitter lives in the TCM
   blob, not the main blob, so it never appeared in hif_alloc_msg_to_host's
   caller list when that list was enumerated from the main program alone.
   
   Known senders (code = first argument):
     code 3   dbg_accumulate_wake_stats (0x00013130), 0x93 bytes of wake statistics
     code 8   txp_pipe_tx_start        (0x00009DEA), 0x18 bytes
     code 10  phy_watchdog_check       (0x0001734E), via hif_send_debug_u32
   
   hif_send_debug_u32 (0xFFF019C8) is the convenience wrapper for a single 4-byte
   payload.
   
   mainline cw1200 has no 0x080E case, so it currently falls to `default: pr_warn`
   -- harmless, and adding a handler is purely additive.  Worth doing: this is a
   firmware-pushed trace stream that needs no polling, unlike the TCM ring buffer at
   0xFFF0364C which must be read with WSM 0x0000.
   
   Lesson for the inventory method: enumerating a helper's callers is only complete
   within one program.  The TCM blob is a separate Ghidra program and its calls into
   main-blob helpers appear as `func_0x...` thunks, invisible to a caller query run
   on either program alone. */

longlong hif_send_debug_msg(uint param_1,undefined4 param_2,int param_3)

{
  undefined2 *puVar1;
  
  puVar1 = (undefined2 *)func_0x0000e63c(param_3 + 8U & 0xffff);
  if (puVar1 != (undefined2 *)0x0) {
    puVar1[1] = (short)DAT_fff019d4;
    *puVar1 = (short)(param_3 + 8U);
    *(uint *)(puVar1 + 2) = param_1;
    func_0x0000f0fc(puVar1 + 4,param_2,param_3);
    func_0x0000eda4(puVar1);
    return (ulonglong)param_1 << 0x20;
  }
  return CONCAT44(param_1,0xffffffff);
}



/* ======================================================================
 * fff019c8  hif_send_debug_u32
 * ====================================================================== */

void hif_send_debug_u32(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined4 uStack_c;
  
  uStack_c = param_2;
  hif_send_debug_msg(param_1,&uStack_c,4,param_4,param_1);
  return;
}



/* ======================================================================
 * fff019f2  uart_hw_init
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void uart_hw_init(void)

{
  _DAT_0a900004 = 0x2b;
  func_0x0001619c(0,PTR_LAB_fff019e0_1_fff01a70);
  func_0x0001619c(2,PTR_LAB_fff019d8_1_fff01a74);
  return;
}



/* ======================================================================
 * fff01a0e  uart_hw_putc
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

undefined4 uart_hw_putc(undefined4 param_1)

{
  int iVar1;
  int iVar2;
  
  iVar2 = DAT_fff01a78;
  iVar1 = DAT_fff01a6c;
  if (*(int *)(DAT_fff01a78 + 0x10) == 0) {
    if ((*(uint *)(DAT_fff01a6c + 0x28) & 1) == 0) {
      func_0x0000efdc();
      *(uint *)(iVar1 + 0xc) = *(uint *)(iVar1 + 0xc) | 1;
      func_0x0000eff0();
      return 0;
    }
    *(undefined4 *)(DAT_fff01a78 + 0x10) = 0x10;
  }
  *(int *)(iVar2 + 0x10) = *(int *)(iVar2 + 0x10) + -1;
  _DAT_0a90000c = param_1;
  return 1;
}



/* ======================================================================
 * fff01a48  uart_hw_getc
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

undefined4 uart_hw_getc(undefined1 *param_1)

{
  if ((_DAT_0a900008 & 0xf) != 0) {
    *param_1 = (char)_DAT_0a900010;
    return 1;
  }
  return 0;
}



/* ======================================================================
 * fff01a5e  uart_noop
 * ====================================================================== */

void uart_noop(void)

{
  return;
}



/* ======================================================================
 * fff01a60  uart_hw_reinit
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void uart_hw_reinit(void)

{
  _DAT_0a900004 = 0x2b;
  return;
}


