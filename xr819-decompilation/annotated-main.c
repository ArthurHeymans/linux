/*
 * Ghidra decompiler export for xr819-annotated-main / ghidra-fw-main.bin
 * Functions: 757
 * Reference pseudocode; not buildable source.
 */


/* ======================================================================
 * 00000000  Reset
 * ====================================================================== */

void Reset(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 uStack_4c;
  undefined4 uStack_48;
  undefined4 uStack_44;
  undefined4 uStack_40;
  undefined4 uStack_3c;
  
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  puVar3 = &uStack_4c;
  uStack_4c = 3;
  uVar4 = 0x16650;
  uStack_48 = param_1;
  uStack_44 = param_2;
  uStack_40 = param_3;
  uStack_3c = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00000004  UndefinedInstruction
 * ====================================================================== */

void UndefinedInstruction
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  int in_lr;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  puStack00000014 = &uStack_20;
  iStack0000001c = in_lr + -4;
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar3 = &uStack_24;
  uStack_24 = 0;
  uVar4 = 0x16650;
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00000008  SupervisorCall
 * ====================================================================== */

void SupervisorCall(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 uStack_4c;
  undefined4 uStack_48;
  undefined4 uStack_44;
  undefined4 uStack_40;
  undefined4 uStack_3c;
  
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  puVar3 = &uStack_4c;
  uStack_4c = 3;
  uVar4 = 0x16650;
  uStack_48 = param_1;
  uStack_44 = param_2;
  uStack_40 = param_3;
  uStack_3c = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 0000000c  PrefetchAbort
 * ====================================================================== */

void PrefetchAbort(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  int in_lr;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  puStack00000014 = &uStack_20;
  iStack0000001c = in_lr + -4;
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar3 = &uStack_24;
  uStack_24 = 1;
  uVar4 = 0x16650;
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00000010  DataAbort
 * ====================================================================== */

void DataAbort(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  int in_lr;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 *puStack00000014;
  int iStack00000018;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  iStack00000018 = in_lr + -4;
  puStack00000014 = &uStack_20;
  iStack0000001c = in_lr + -8;
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar3 = &uStack_24;
  uStack_24 = 2;
  uVar4 = 0x16650;
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00000014  NotUsed
 * ====================================================================== */

void NotUsed(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 uStack_4c;
  undefined4 uStack_48;
  undefined4 uStack_44;
  undefined4 uStack_40;
  undefined4 uStack_3c;
  
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  puVar3 = &uStack_4c;
  uStack_4c = 3;
  uVar4 = 0x16650;
  uStack_48 = param_1;
  uStack_44 = param_2;
  uStack_40 = param_3;
  uStack_3c = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00000018  IRQ
 * ====================================================================== */

undefined8 IRQ(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int in_lr;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  hif_irq_demux(in_lr + -4,&uStack_20);
  return CONCAT44(uStack_1c,uStack_20);
}



/* ======================================================================
 * 0000001c  FIQ
 * ====================================================================== */

undefined8 FIQ(undefined4 param_1,undefined4 param_2)

{
  mac_irq_handler();
  return CONCAT44(param_2,param_1);
}



/* ======================================================================
 * 0000002c  irq_vector_entry
 * ====================================================================== */

undefined8
irq_vector_entry(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int in_lr;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  hif_irq_demux(in_lr + -4,&uStack_20);
  return CONCAT44(uStack_1c,uStack_20);
}



/* ======================================================================
 * 00000044  lmc_init_link_tables
 * ====================================================================== */

void lmc_init_link_tables(void)

{
  undefined4 *puVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  puVar1 = DAT_000000a8;
  *DAT_000000a8 = 0;
  *puVar1 = 0x1000000;
  iVar2 = DAT_000000ac;
  uVar3 = 0;
  do {
    iVar4 = uVar3 * 0xc + iVar2;
    *(undefined4 *)(iVar4 + 0x20) = 0xffffffff;
    *(undefined4 *)(iVar4 + 0x24) = 0xff;
    *(undefined4 *)(iVar4 + 0x28) = 0xff;
    iVar4 = DAT_000000b0;
    uVar3 = uVar3 + 1 & 0xff;
  } while (uVar3 < 4);
  uVar3 = 0;
  do {
    *(undefined1 *)(iVar4 + uVar3 + 0x64e) = 0;
    timer_entry_init(uVar3 * 0x38 + iVar4 + 0x678,DAT_000000b4,uVar3);
    uVar3 = uVar3 + 1 & 0xff;
  } while (uVar3 < 8);
  *(undefined2 *)(DAT_000000b8 + 0x18) = 0;
  link_state_init_all();
  return;
}



/* ======================================================================
 * 000000bc  mac_hw_reset_regs
 * ====================================================================== */

void mac_hw_reset_regs(void)

{
  undefined4 *puVar1;
  
  *DAT_00000124 = 0;
  *(undefined4 *)(DAT_00000128 + 0xc) = 0;
  puVar1 = DAT_0000012c;
  DAT_0000012c[3] = 0x80;
  *puVar1 = DAT_00000130;
  puVar1[7] = 7;
  puVar1[1] = 0;
  puVar1[2] = 0;
  puVar1[8] = (int)puVar1 << 0x13;
  puVar1 = DAT_00000134;
  *DAT_00000134 = 0;
  puVar1[1] = 0;
  puVar1[3] = 3;
  *DAT_00000138 = 0;
  puVar1 = DAT_00000140;
  *DAT_00000140 = DAT_0000013c;
  puVar1[1] = 0;
  puVar1[7] = 0;
  return;
}



/* ======================================================================
 * 000000f6  fw_subsystem_init
 * ====================================================================== */

void fw_subsystem_init(void)

{
  mac_hw_reset_regs();
  tsf_hw_init();
  mac_hw_init_pipes();
  lmc_init_link_tables();
  hif_dbg_ctx_init();
  hif_rx_bufs_init();
  irq_register_handler(0x1b,DAT_00000144);
  irq_register_handler(0x1a,DAT_00000148);
  return;
}



/* ======================================================================
 * 000002e6  mac_rx_pause
 * ====================================================================== */

void mac_rx_pause(void)

{
  *(uint *)(DAT_00000388 + 0xc) = *(uint *)(DAT_00000388 + 0xc) & 0xffffffc3;
  return;
}



/* ======================================================================
 * 000002f4  mac_rx_restart
 * ====================================================================== */

void mac_rx_restart(void)

{
  undefined4 uVar1;
  
  uVar1 = mac_rx_pause();
  phy_rx_disable_and_drain();
  phy_rx_enable();
  *(undefined4 *)(DAT_00000388 + 0xc) = uVar1;
  return;
}



/* ======================================================================
 * 0000030a  mac_rx_pause_briefly
 * ====================================================================== */

void mac_rx_pause_briefly(void)

{
  undefined4 uVar1;
  
  uVar1 = mac_rx_pause();
  fw_delay_loop(1);
  *(undefined4 *)(DAT_00000388 + 0xc) = uVar1;
  return;
}



/* ======================================================================
 * 0000038c  txp_fn_4425
 * ====================================================================== */

undefined8 txp_fn_4425(int param_1,uint param_2,int param_3)

{
  byte *pbVar1;
  byte bVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  byte *pbVar7;
  uint uVar8;
  int iVar9;
  uint local_20;
  uint local_1c;
  undefined4 local_18;
  
  local_18 = 1;
  iVar3 = mac_hw_idle();
  local_20 = param_2;
  if ((iVar3 == 0) && (iVar3 = tx_pipes_all_idle(), iVar3 == 0)) {
    if (param_1 == 0) {
LAB_000003de:
      if (param_3 != 0) {
LAB_000004b8:
        evt_flags_set(DAT_0000051c,0x200000);
        return CONCAT44(local_20,local_18);
      }
      goto LAB_000003e2;
    }
    iVar3 = fw_read_timer();
    local_18 = 2;
    do {
      iVar4 = fw_read_timer();
      if ((int)param_2 < iVar4 - iVar3) goto LAB_000003de;
      iVar4 = mac_hw_idle();
    } while ((iVar4 == 0) && (iVar4 = tx_pipes_all_idle(), iVar4 == 0));
  }
  local_18 = 0;
LAB_000003e2:
  local_1c = 0;
LAB_000003e6:
  iVar4 = local_1c * 0x6c + DAT_00000504;
  pbVar7 = (byte *)(iVar4 + 0xa0);
  iVar3 = txp_pipe_advance_slot(local_1c);
  if (iVar3 != 0) {
    uVar8 = (uint)*(byte *)(iVar4 + 0xa2);
    do {
      iVar3 = *(int *)(pbVar7 + uVar8 * 0x18 + 0x18);
      pbVar1 = pbVar7 + uVar8 * 0x18 + 0x18;
      pbVar1[0] = 0;
      pbVar1[1] = 0;
      pbVar1[2] = 0;
      pbVar1[3] = 0;
      pbVar7[uVar8 * 0x18 + 0xf] = 0;
      if (iVar3 == 0) {
LAB_0000045c:
        if ((pbVar7[uVar8 * 0x18 + 0xc] == 1) && (*(int *)(pbVar7 + uVar8 * 0x18 + 0x1c) != 0)) {
          desc_freelist_push();
        }
      }
      else {
        bVar2 = pbVar7[uVar8 * 0x18 + 0xc];
        if (bVar2 < 2) {
          if (bVar2 == 1) {
            local_20 = (uint)*(byte *)(iVar3 + 0x6c);
            link_set_state(local_20,5);
            uVar5 = 0;
            iVar9 = local_20 * 0x40 + DAT_00000504;
            do {
              iVar6 = uVar5 * 4;
              uVar5 = uVar5 + 1 & 0xff;
              *(undefined4 *)(iVar6 + iVar9 + 0x490) = 0;
            } while (uVar5 < 0x10);
          }
          do {
            *(undefined2 *)(iVar3 + 0x1c) = 0x18;
            tx_ctx_free_inner(iVar3);
            iVar3 = *(int *)(iVar3 + 0x3c);
          } while (iVar3 != 0);
          goto LAB_0000045c;
        }
      }
      if (*(byte *)(iVar4 + 0xa1) == uVar8) goto LAB_0000047a;
      uVar8 = uVar8 + 1 & 3;
    } while( true );
  }
  goto LAB_00000486;
LAB_0000047a:
  *(char *)(iVar4 + 0xa2) = *(char *)(iVar4 + 0xa1);
  *pbVar7 = *(char *)(iVar4 + 0xa1) + 1U & 3;
LAB_00000486:
  *(byte *)(iVar4 + 0xa4) = *(byte *)(iVar4 + 0xa4) & 0xf6;
  *(undefined1 *)(iVar4 + 0xa5) = 5;
  if ((uint)*pbVar7 != (*(uint *)(*(int *)(iVar4 + 0xa8) + 0x20) & 0x3fffffff) >> 0x1b) {
    fw_assert(s_tx_ptcs_c_00000510,DAT_0000050c,DAT_00000508);
  }
  local_1c = local_1c + 1 & 0xff;
  if (3 < local_1c) goto LAB_000004b8;
  goto LAB_000003e6;
}



/* ======================================================================
 * 000004c6  hif_rx_bufs_init
 * ====================================================================== */

void hif_rx_bufs_init(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  int *piVar4;
  
  piVar4 = (int *)(DAT_00000520 + -4);
  *piVar4 = DAT_00000520;
  iVar2 = DAT_00000528;
  iVar1 = DAT_00000524;
  uVar3 = 0;
  do {
    piVar4[uVar3 * 2 + 2] = uVar3 * 0x2a0 + iVar2 + iVar1;
    if (uVar3 == 3) {
      *(undefined4 *)(DAT_00000520 + 0x18) = 0;
    }
    else {
      piVar4[uVar3 * 2 + 1] = (int)(piVar4 + uVar3 * 2 + 3);
    }
    uVar3 = uVar3 + 1 & 0xff;
  } while (uVar3 < 4);
  return;
}



/* ======================================================================
 * 0000052c  rx_subsystem_init
 * ====================================================================== */

void rx_subsystem_init(void)

{
  mac_program_base_regs();
  mac_program_timing_regs();
  return;
}



/* ======================================================================
 * 000005a0  sched_clear_pending
 * ====================================================================== */

void sched_clear_pending(void)

{
  *(undefined2 *)(DAT_00000710 + 0x14) = 0;
  return;
}



/* ======================================================================
 * 000005a8  fw_timers_and_tasks_init
 * ====================================================================== */

/* fw_timers_and_tasks_init() -- the master timer and scheduler-task registration.
   Read this to find which event bit drives which subsystem.
   
   Per-vif timers, registered for vifs 0..1 (5 each, all passed vif+0x18 as ctx):
     vif+0x0C4  vif+0x0B0  vif+0x0D8  vif+0x184  vif+0x198
   
   Then a set of global timers, tx_ctx_pool_init(), and eight scheduler tasks:
   
     event bit      handler DAT        subsystem (from the handlers now named)
     0x80000000     DAT_00000758
     0x00200000     DAT_0000075C       TX confirm / completion sweep
     0x00008000     DAT_00000760
     0x00002000     DAT_00000764       802.11k measurement (set by measure_start)
     0x00000200     DAT_00000740
     0x00001000     DAT_00000768       channel switch (set by chanswitch_request)
     0x00000400     DAT_00000730       scan state machine (set by syn_scan_*)
     0x00000100     DAT_0000076C
   
   Cross-reference for the event bits used elsewhere in this project:
     0x00000040   beacon / TBTT          (beacon_schedule_next)
     0x00000400   scan                   (syn_scan_try_start / _abort / _complete)
     0x00001000   channel switch         (chanswitch_request)
     0x00002000   measurement            (measure_start)
     0x00040000   PHY state change       (phy_state_advance)
     0x00080000   TX buffer low          (txbuf_freelist_pop, >16 outstanding)
     0x00100000   HIF confirm flush      (gated by hif_confirm_coalesce_hold)
     0x00200000   TX completion          (bab_process_ba_bitmap, ps paths)
     0x00400000   host message queued    (lmc_msg_alloc consumers)
     0x08000000   scheduler timer due    (sched_arm_next_timer)
   
   Note vif index 2 gets its if_id byte set (+0x1A) in the first loop over 3 vifs
   but NO timers -- consistent with if_id 2 being the P2P-device slot, which never
   beacons or scans on its own. */

void fw_timers_and_tasks_init(void)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  iVar1 = DAT_00000704;
  uVar2 = 0;
  do {
    *(char *)(uVar2 * 0x3b0 + iVar1 + 0x1a) = (char)uVar2;
    uVar2 = uVar2 + 1;
  } while (uVar2 < 3);
  uVar2 = 0;
  do {
    iVar3 = uVar2 * 0x3b0 + iVar1;
    iVar4 = iVar3 + 0x18;
    timer_entry_init(iVar3 + 0xc4,DAT_00000714,iVar4);
    timer_entry_init(iVar3 + 0xb0,DAT_00000718,iVar4);
    timer_entry_init(iVar3 + 0xd8,DAT_0000071c,iVar4);
    timer_entry_init(iVar3 + 0x184,DAT_00000720,iVar4);
    timer_entry_init(iVar3 + 0x198,DAT_00000724,iVar4);
    iVar3 = DAT_0000072c;
    uVar2 = uVar2 + 1;
  } while (uVar2 < 2);
  timer_entry_init(DAT_0000072c,DAT_00000728,0);
  timer_entry_init(DAT_00000734,DAT_00000730,0);
  timer_entry_init(DAT_00000734 + 0x14,DAT_00000738,0);
  timer_entry_init(DAT_00000734 + -0x60,DAT_0000073c,0);
  timer_entry_init(DAT_00000734 + 0x28,DAT_00000740,0);
  timer_entry_init(DAT_00000748,DAT_00000744,0);
  *(undefined1 *)(DAT_0000074c + 8) = 0;
  timer_entry_init(DAT_00000734 + 0x3c,DAT_00000750,0);
  sched_clear_pending();
  tx_ctx_pool_init();
  ps_per_vif_timers_init();
  lmc_flush_pending_tx();
  template_frame_table_init();
  mib_defaults_copy();
  dup_cache_init();
  lmc_msg_pool_init();
  bab_init();
  timer_entry_init(DAT_00000734 + -0x44,DAT_00000754,0);
  sched_register_task(0x80000000,DAT_00000758);
  sched_register_task(0x200000,DAT_0000075c);
  sched_register_task(0x8000,DAT_00000760);
  sched_register_task(0x2000,DAT_00000764);
  sched_register_task(0x200,DAT_00000740);
  sched_register_task(0x1000,DAT_00000768);
  sched_register_task(0x400,DAT_00000730);
  sched_register_task(0x100,DAT_0000076c);
  *(undefined1 *)(DAT_0000072c + -1) = 1;
  timer_start(iVar3,DAT_00000770);
  return;
}



/* ======================================================================
 * 0000079c  ps_per_vif_timers_init
 * ====================================================================== */

void ps_per_vif_timers_init(void)

{
  int iVar1;
  undefined4 uVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  
  iVar1 = DAT_00000828;
  uVar5 = 0;
  do {
    uVar2 = DAT_0000082c;
    iVar3 = uVar5 * 0x104 + iVar1;
    iVar4 = iVar3 + 0x40;
    *(char *)(iVar3 + 0x43) = (char)uVar5;
    timer_entry_init(iVar3 + 0x84,uVar2,iVar4);
    timer_entry_init(iVar3 + 0xd4,DAT_00000830,iVar4);
    timer_entry_init(iVar3 + 0x98,DAT_00000834,iVar4);
    timer_entry_init(iVar3 + 0xac,DAT_00000838,iVar4);
    timer_entry_init(iVar3 + 0x70,DAT_0000083c,iVar4);
    timer_entry_init(iVar3 + 0xc0,DAT_00000840,iVar4);
    timer_entry_init(iVar3 + 0xe8,DAT_00000844,iVar4);
    *(undefined **)(iVar3 + 0x118) = &DAT_00001f40;
    *(undefined **)(iVar3 + 0x11c) = &DAT_00001f40;
    *(undefined4 *)(iVar3 + 0x120) = 4000;
    vif_reset_all_state(uVar5 & 0xff);
    uVar5 = uVar5 + 1;
  } while (uVar5 < 2);
  return;
}



/* ======================================================================
 * 00000848  mib_defaults_copy
 * ====================================================================== */

void mib_defaults_copy(void)

{
  fw_memcpy(DAT_00000864,DAT_00000860,0x30);
  *(undefined4 *)((int)DAT_00000864 + 0x50) = 0;
  return;
}



/* ======================================================================
 * 00000868  lmc_msg_pool_init
 * ====================================================================== */

void lmc_msg_pool_init(void)

{
  int iVar1;
  int iVar2;
  undefined1 *puVar3;
  char cVar4;
  
  iVar1 = DAT_000008a0;
  *(undefined4 *)(DAT_000008a0 + 8) = 0;
  iVar2 = DAT_000008a4;
  puVar3 = DAT_0000089c;
  *(undefined4 *)(iVar1 + 4) = 0;
  cVar4 = '\x02';
  while (cVar4 != '\0') {
    *(undefined4 *)(puVar3 + 0x18) = *(undefined4 *)(iVar1 + 4);
    *puVar3 = 0;
    *(undefined1 **)(puVar3 + 0xf4) = puVar3 + 0x1c;
    *(undefined1 **)(iVar1 + 4) = puVar3;
    puVar3 = puVar3 + iVar2;
    cVar4 = cVar4 + -1;
  }
  return;
}



/* ======================================================================
 * 000008a8  template_frame_table_init
 * ====================================================================== */

void template_frame_table_init(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  undefined1 *puVar4;
  int iVar5;
  
  iVar2 = DAT_0000093c;
  iVar1 = DAT_00000938;
  uVar3 = 0;
  do {
    puVar4 = (undefined1 *)(uVar3 * 0x40 + DAT_00000940);
    iVar5 = DAT_00000940 + 0x80;
    *puVar4 = 0;
    puVar4[1] = 0;
    *(uint *)(puVar4 + 4) = uVar3 * 0x100 + iVar5;
    puVar4[8] = 1;
    iVar5 = DAT_00000944;
    puVar4[9] = 0;
    iVar5 = uVar3 * iVar5 + iVar2;
    *(int *)(puVar4 + 0xc) = iVar5;
    puVar4[0x28] = 5;
    puVar4[0x29] = 0;
    *(int *)(puVar4 + 0x2c) = iVar5 + iVar1;
    puVar4[0x10] = 2;
    puVar4[0x11] = 0xff;
    *(undefined2 *)(puVar4 + 0x12) = 0x18;
    puVar4[0x18] = 3;
    puVar4[0x19] = 0xff;
    *(undefined2 *)(puVar4 + 0x1a) = 0x1a;
    puVar4[0x20] = 4;
    puVar4[0x21] = 0xff;
    *(undefined2 *)(puVar4 + 0x22) = 0x10;
    iVar5 = DAT_00000948;
    *(uint *)(puVar4 + 0x34) = uVar3 * 0x60 + DAT_00000948;
    puVar4[0x30] = 6;
    *(uint *)(puVar4 + 0x3c) = uVar3 * 0x90 + iVar5 + 0xc0;
    puVar4[0x38] = 7;
    puVar4[0x39] = 0xd;
    uVar3 = uVar3 + 1;
    *(undefined2 *)(puVar4 + 0x3a) = 0x90;
  } while (uVar3 < 2);
  return;
}



/* ======================================================================
 * 0000094c  hif_host_bufs_init
 * ====================================================================== */

/* hif_host_bufs_init() -- allocate and register the host-facing WSM buffers.
   
   *** THIS IS THE AUTHORITATIVE SOURCE OF input_buffers AND size_inp_ch_buf. ***
   Both values the startup indication reports to the host are plain hardcoded
   constants here, not computed from memory size or negotiated:
   
     for (i = 0; i < 4;    i++) g_rx_q[i+2] = base + i*0x180;   /* 4 x 384  */
     for (i = 0; i < 0x1E; i++) list[i]     = base + i*0x660;   /* 30 x 1632 */
     hif_init(list, 0x1E, 0x660);
   
     0x1E  = 30    == wsm_caps.input_buffers
     0x660 = 1632  == wsm_caps.input_buffer_size
   
   hif_init (0x00000ACA) stores the size into the HIF control block at +0x29,
   queues all 30 buffers to the host with hif_queue_msg_to_host, and asserts:
   
     if (count > 0x20) fw_assert("hif.c", 0x129, 3);
   
   **So 32 is the hard architectural ceiling and 30 is the shipped value** -- there
   are two spare slots, but nothing in the firmware exposes a way to change the
   count at runtime.  That closes the question of whether WSM 0x0023
   REQUEST_BUFFER could raise the host's credit ceiling: it clamps to 30, and even
   the internal limit is 32.
   
   Note these 1632-byte buffers are the WSM *message* buffers (the host writes
   requests into them).  They are distinct from the tx_ctx descriptor pools --
   see tx_ctx_pool_init (3 internal descriptors) and tx_wsm_buf_alloc (host data
   path, head 0x040087B0). */

void hif_host_bufs_init(void)

{
  uint *puVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  uint uVar5;
  int local_84 [30];
  
  iVar3 = DAT_000009a4;
  iVar2 = DAT_000009a0;
  puVar1 = DAT_0000099c;
  uVar5 = 0;
  do {
    uVar4 = uVar5 + 1;
    puVar1[uVar5 + 2] = uVar5 * 0x180 + iVar3 + iVar2;
    uVar5 = uVar4;
  } while (uVar4 < 4);
  *puVar1 = uVar4;
  uVar5 = 0;
  do {
    uVar4 = uVar5 + 1;
    local_84[uVar5] = uVar5 * 0x660 + iVar3 + DAT_000009a8;
    uVar5 = uVar4;
  } while (uVar4 < 0x1e);
  hif_init(local_84,0x1e,0x660);
  return;
}



/* ======================================================================
 * 00000aca  hif_init
 * ====================================================================== */

void hif_init(int param_1,uint param_2,undefined2 param_3)

{
  int iVar1;
  int *piVar2;
  int iVar3;
  uint uVar4;
  
  if (0x20 < param_2) {
    fw_assert(s_hif_c_00000b68,0x129,3);
  }
  sched_register_task(0x40000000,DAT_00000b70);
  sched_register_task(0x2000000,DAT_00000b74);
  piVar2 = DAT_00000b64;
  *(undefined2 *)(DAT_00000b64 + 0x29) = param_3;
  piVar2[0x6f] = 0xab00000;
  piVar2[0x6e] = 0x1f;
  iVar3 = DAT_00000b78;
  *(int *)(DAT_00000b78 + 0xc) = DAT_00000b60 + -0x40;
  *(undefined4 *)(iVar3 + 8) = 3;
  for (uVar4 = 0; uVar4 < param_2; uVar4 = uVar4 + 1) {
    hif_queue_msg_to_host(*(undefined4 *)(param_1 + uVar4 * 4));
  }
  irq_register_handler(0xd,DAT_00000b7c);
  uVar4 = DAT_00000b80;
  iVar1 = DAT_00000b60;
  *(uint *)(DAT_00000b60 + -0x18) = DAT_00000b80;
  if ((*DAT_00000b64 == 1) && (*(int *)(iVar1 + -0x20) << 0x15 < 0)) {
    uVar4 = DAT_00000b80 - 8;
  }
  *(uint *)(iVar3 + 0x10) = uVar4 & 0x7f8;
  *(uint *)(iVar1 + -0x20) = uVar4;
  timer_entry_init(DAT_00000b64 + 3,DAT_00000b84,0);
  return;
}



/* ======================================================================
 * 00000b88  hif_register_irq_and_task
 * ====================================================================== */

void hif_register_irq_and_task(void)

{
  irq_register_handler(0x15,DAT_00000ba0);
  sched_register_task(0x20000000,DAT_00000ba4);
  return;
}



/* ======================================================================
 * 00000c88  bab_init
 * ====================================================================== */

void bab_init(void)

{
  int iVar1;
  uint uVar2;
  
  iVar1 = DAT_00000cbc;
  uVar2 = 0;
  do {
    timer_entry_init(uVar2 * 0x28 + iVar1 + 0x3b4,DAT_00000cc0,uVar2);
    uVar2 = uVar2 + 1;
  } while (uVar2 < 4);
  sched_register_task(0x400000,DAT_00000cc4);
  *(undefined1 *)(DAT_00000cc8 + 0x1b) = 1;
  return;
}



/* ======================================================================
 * 00000ccc  tsf_add_offset
 * ====================================================================== */

void tsf_add_offset(int param_1)

{
  int iVar1;
  
  do {
    iVar1 = *(int *)(DAT_00000d7c + 0x3c);
  } while (*(int *)(DAT_00000d7c + 0x3c) != iVar1);
  if ((param_1 < 0) && (-1 < *(int *)(DAT_00000d7c + 0x38))) {
    iVar1 = iVar1 + -1;
  }
  u64_add_u32(0,iVar1,param_1);
  return;
}



/* ======================================================================
 * 00000cf0  tsf_write
 * ====================================================================== */

void tsf_write(undefined4 param_1,undefined4 param_2)

{
  int iVar1;
  
  irq_fiq_disable_save();
  iVar1 = DAT_00000d7c;
  *(undefined4 *)(DAT_00000d7c + 0x34) = 1;
  *(undefined4 *)(iVar1 + 0x38) = param_1;
  *(undefined4 *)(iVar1 + 0x3c) = param_2;
  *(undefined4 *)(iVar1 + 0x34) = 3;
  irq_fiq_restore();
  return;
}



/* ======================================================================
 * 00000d0e  tsf_adjust_small
 * ====================================================================== */

void tsf_adjust_small(uint param_1,int param_2,int param_3)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  
  iVar1 = DAT_00000d7c;
  if ((*(int *)(DAT_00000d7c + 0x3c) == param_2) &&
     ((uVar3 = param_1 - param_3, uVar3 < 0x100 || (0xfffffeff < uVar3)))) {
    *(uint *)(DAT_00000d80 + 0x3c) = uVar3;
    irq_disable_save();
    *(uint *)(iVar1 + 0x34) = (uVar3 & 0x1ff) * 0x100 + 7;
    irq_restore();
    return;
  }
  iVar2 = (*(int *)(DAT_00000d7c + 0x38) - param_3) + 6;
  uVar4 = param_1 + iVar2;
  *(int *)(DAT_00000d80 + 0x3c) = iVar2;
  uVar3 = uVar4;
  if (iVar2 < 1) {
    if ((param_1 < uVar4) && (uVar3 = param_1, param_2 != 0)) {
      param_2 = param_2 + -1;
      uVar3 = uVar4;
    }
  }
  else if (uVar4 < param_1) {
    param_2 = param_2 + 1;
  }
  irq_fiq_disable_save();
  *(undefined4 *)(iVar1 + 0x34) = 1;
  *(uint *)(iVar1 + 0x38) = uVar3;
  *(int *)(iVar1 + 0x3c) = param_2;
  *(undefined4 *)(iVar1 + 0x34) = 3;
  irq_fiq_restore();
  return;
}



/* ======================================================================
 * 00000d84  phy_wake_sequence
 * ====================================================================== */

void phy_wake_sequence(void)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined4 local_10;
  
  iVar1 = DAT_00000f88;
  local_10 = 0;
  if (*(char *)(DAT_00000f88 + 0x1c) != '\x02') {
    iVar3 = phy_cal_run_step_timed(*(char *)(DAT_00000f88 + 0x1c),0,&local_10);
    iVar2 = DAT_00000f8c;
    *(char *)(iVar1 + 0x1c) = (char)iVar3;
    if ((int)((uint)*(byte *)(iVar2 + 1) << 0x1e) < 0) {
      *(byte *)(iVar2 + 1) = *(byte *)(iVar2 + 1) & 0xfd | 4;
    }
    if (iVar3 != 2) {
      mac_reinit_after_wake();
      iVar3 = phy_cal_run_step_timed(*(undefined1 *)(iVar1 + 0x1c),0,&local_10);
      *(char *)(iVar1 + 0x1c) = (char)iVar3;
      if (iVar3 != 2) {
        timer_start(DAT_00000f88 + 8,local_10);
        return;
      }
    }
    if ((*(byte *)(iVar2 + 1) & 1) != 0) {
      *(byte *)(iVar2 + 1) = *(byte *)(iVar2 + 1) & 0xfe;
    }
    if ((-1 < (int)((uint)*(byte *)(iVar2 + 1) << 0x1d)) && (*(char *)(iVar1 + 0x1d) != '\0')) {
      mac_reprogram_after_channel();
    }
    phy_resume_state4();
  }
  return;
}



/* ======================================================================
 * 00000df4  task_df4
 * ====================================================================== */

void task_df4(void)

{
  if (*(char *)(DAT_00000f8c + 0xb) != '\0') {
    *(undefined1 *)(DAT_00000f8c + 0xb) = 0;
    pac_phy_stop_op();
    thunk_16b4e();
    *(undefined1 *)(DAT_00000f90 + 0x16) = 1;
    evt_flags_clear(0x40000);
  }
  return;
}



/* ======================================================================
 * 00000e18  tx_flush_all_queues
 * ====================================================================== */

void tx_flush_all_queues(void)

{
  byte bVar1;
  int iVar2;
  uint *puVar3;
  undefined4 uVar4;
  uint uVar5;
  int iVar6;
  int iVar7;
  uint uVar8;
  uint uVar9;
  int iVar10;
  
  iVar2 = DAT_00000f90;
  bVar1 = *(byte *)(DAT_00000f90 + 0x15);
  *(byte *)(DAT_00000f90 + 0x15) = bVar1 | 1;
  wait_pipes_idle(DAT_00000f94,1);
  uVar4 = irq_fiq_disable_save();
  puVar3 = DAT_00000f98;
  uVar5 = DAT_00000f98[1];
  uVar8 = *DAT_00000f98 & 0xff;
  for (uVar9 = *DAT_00000f98 & 0xff; uVar9 != (uVar5 & 0xff); uVar9 = uVar9 + 1 & 0x3f) {
    if (puVar3[uVar9 + 2] != 0) {
      *(undefined2 *)(puVar3[uVar9 + 2] + 0x1c) = 0x14;
      tx_ctx_free_inner();
      puVar3[uVar9 + 2] = 0;
      evt_flags_set(DAT_00000f9c,0x100000);
    }
  }
  while ((uVar8 != (uVar5 & 0xff) && (puVar3[uVar8 + 2] == 0))) {
    uVar8 = uVar8 + 1 & 0x3f;
    *puVar3 = uVar8;
  }
  uVar5 = 0;
  do {
    uVar8 = 0;
    iVar10 = uVar5 * 0x40 + DAT_00000f8c;
    do {
      iVar6 = iVar10 + uVar8 * 4;
      iVar7 = *(int *)(iVar6 + 0x490);
      if (iVar7 != 0) {
        *(undefined2 *)(iVar7 + 0x1c) = 0x14;
        tx_ctx_free_inner();
        *(undefined4 *)(iVar6 + 0x490) = 0;
      }
      uVar8 = uVar8 + 1 & 0xff;
    } while (uVar8 < 0x10);
    uVar5 = uVar5 + 1 & 0xff;
  } while (uVar5 < 8);
  irq_fiq_restore(uVar4);
  *(byte *)(iVar2 + 0x15) = bVar1;
  return;
}



/* ======================================================================
 * 00000ee4  tx_assign_seq_num
 * ====================================================================== */

void tx_assign_seq_num(int *param_1)

{
  ushort uVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  
  if (param_1[1] << 2 < 0) {
    uVar4 = (uint)*(byte *)((uint)*(byte *)((int)param_1 + 0x69) * 0x3b0 + DAT_00000fa0 + 0x12a);
    if (*(char *)((int)param_1 + 0x6b) != '\0') {
      for (iVar2 = 0; iVar2 < (int)(uint)*(ushort *)(DAT_00000fa4 + 0x14); iVar2 = iVar2 + 1) {
        iVar3 = iVar2 * 0xc + DAT_00000fa0 + DAT_00000fa8;
        if (((uint)*(byte *)(iVar3 + 0x19) == (uint)*(byte *)((int)param_1 + 0x69)) &&
           (*(char *)(iVar3 + 0x18) == *(char *)((int)param_1 + 0x6b))) {
          uVar4 = (uint)*(byte *)(iVar2 * 0xc + DAT_00000fa0 + DAT_00000fa8 + 0x1a);
          break;
        }
      }
    }
    iVar2 = uVar4 * 0x20 + DAT_00000fa0 + (*(ushort *)(*param_1 + 0x18) & 0xf) * 2 +
            DAT_00000fa8 + 0xc0;
    uVar1 = *(ushort *)(iVar2 + 0x18);
    *(ushort *)(*param_1 + 0x16) = uVar1;
    *(ushort *)(param_1 + 0x15) = uVar1 >> 4;
    *(ushort *)(iVar2 + 0x18) = uVar1 + 0x10 & 0xfff0;
  }
  return;
}



/* ======================================================================
 * 00000f74  phy_set_cfg_word
 * ====================================================================== */

void phy_set_cfg_word(undefined4 param_1)

{
  *(undefined4 *)(DAT_00000fac + 0xc) = param_1;
  return;
}



/* ======================================================================
 * 00000f7a  phy_restore_cfg_if_mode2
 * ====================================================================== */

void phy_restore_cfg_if_mode2(uint param_1)

{
  if (param_1 >> 0x1e == 2) {
    *(undefined4 *)(DAT_00000fac + 0x10) = *(undefined4 *)(DAT_00000fac + 0x14);
  }
  return;
}



/* ======================================================================
 * 00000fb0  ofdm_calc_duration
 * ====================================================================== */

uint ofdm_calc_duration(uint param_1,int param_2)

{
  int iVar1;
  
  if (param_1 < 6) {
    return 0;
  }
  iVar1 = __udivsi3(param_2 * 8 + (uint)*(ushort *)(DAT_00000fdc + (param_1 - 6 & 0xf) * 2) + 0x15);
  return iVar1 * 3 + 9U & 0xfff;
}



/* ======================================================================
 * 00000fe0  fw_rand_masked
 * ====================================================================== */

ushort fw_rand_masked(ushort param_1)

{
  ushort uVar1;
  
  uVar1 = fw_rand24_lfsr();
  return uVar1 & param_1;
}



/* ======================================================================
 * 00000ff0  txp_build_tbtt_desc
 * ====================================================================== */

void txp_build_tbtt_desc(uint *param_1,int param_2,int param_3)

{
  short sVar1;
  uint uVar2;
  
  uVar2 = *(int *)(DAT_000010d0 + 0x30) + *(int *)(DAT_000010d0 + 0x1c) * param_2 * 8 & 0x1fff;
  *param_1 = uVar2 | uVar2 << 0x10 | 0x20000000;
  if (param_3 == 0) {
    sVar1 = 0;
  }
  else {
    sVar1 = ((ushort)param_3 & 0x3ff) + 0xdc00;
  }
  *(short *)((int)param_1 + 6) = sVar1;
  param_1[2] = 0x80000000;
  return;
}



/* ======================================================================
 * 0000102a  event_send_error_0x34
 * ====================================================================== */

void event_send_error_0x34(void)

{
  undefined4 local_10;
  undefined4 local_c;
  
  local_10 = 0;
  local_c = 0x34;
  ind_0805_event_a(0,&local_10);
  return;
}



/* ======================================================================
 * 000010dc  txp_desc_set_rate
 * ====================================================================== */

void txp_desc_set_rate(int param_1,int param_2)

{
  uint uVar1;
  
  uVar1 = pas_rate_to_hw_code(*(undefined1 *)(param_2 + 0xf));
  *(uint *)(param_1 + 8) = (uVar1 & 0xff) << 0x10 | (*(uint *)(param_1 + 8) & 0xffff) + 0x52000000;
  return;
}



/* ======================================================================
 * 000010fe  txp_desc_set_rate2
 * ====================================================================== */

void txp_desc_set_rate2(int param_1,int param_2)

{
  uint uVar1;
  
  *(uint *)(param_1 + 0xc) =
       *(uint *)(param_1 + 0xc) & 0xfffffff0 |
       (uint)*(byte *)(DAT_0000129c + (uint)*(byte *)(param_2 + 0xf));
  uVar1 = pas_rate_to_hw_code(*(undefined1 *)(param_2 + 0xf));
  *(uint *)(param_1 + 0x14) =
       (uVar1 & 0xff) << 0x10 | (*(uint *)(param_1 + 0x14) & 0xffff) + 0x52000000;
  return;
}



/* ======================================================================
 * 00001132  txp_build_pipe_words
 * ====================================================================== */

void txp_build_pipe_words(int *param_1,int *param_2,uint param_3)

{
  uint uVar1;
  int iVar2;
  uint local_28;
  uint local_24;
  int *piStack_20;
  int *piStack_1c;
  uint local_18;
  
  piStack_20 = param_1;
  piStack_1c = param_2;
  local_18 = param_3;
  iVar2 = pas_rate_to_hw_code((char)param_2[0x16]);
  pas_build_phy_rate_words
            (&local_24,&local_28,(char)param_2[0x16],0,*(undefined1 *)((int)param_2 + 0xd));
  *param_1 = (local_28 & 0xffffff) + 0x51000000;
  param_1[1] = (local_24 & 0xffffff) + 0x50000000;
  uVar1 = DAT_000012a0;
  param_1[3] = DAT_000012a4;
  param_1[4] = 0x47000000;
  param_1[2] = iVar2 << 0x10 | uVar1;
  param_1[5] = ((uint)*(byte *)((int)param_2 + 0x69) + DAT_000012a8 & 0x7fffff) + 0x20800000;
  param_1[6] = (local_18 & 0xffffff) + 0x32000000;
  param_1[7] = (*param_2 + 4U & 0x7fffff) + 0x26000000;
  param_1[8] = DAT_000012ac;
  param_1[9] = -0x10000000;
  return;
}



/* ======================================================================
 * 000011c4  txp_build_pipe_words_rts
 * ====================================================================== */

void txp_build_pipe_words_rts(int *param_1,int param_2,uint param_3)

{
  int iVar1;
  int iVar2;
  uint local_30;
  uint local_2c;
  int local_28;
  int local_24;
  int *local_20;
  int iStack_1c;
  uint local_18;
  
  local_24 = param_2 + 0x60;
  iVar2 = (uint)*(byte *)(param_2 + 0x69) * 0x98 + DAT_000012b0;
  local_20 = param_1;
  iStack_1c = param_2;
  local_18 = param_3;
  local_28 = pas_rate_to_hw_code(*(undefined1 *)((uint)*(byte *)(param_2 + 0x58) + iVar2 + 0x494));
  pas_build_phy_rate_words
            (&local_2c,&local_30,*(undefined1 *)((uint)*(byte *)(param_2 + 0x58) + iVar2 + 0x494),0,
             *(undefined1 *)(param_2 + 0xd));
  *local_20 = (local_30 & 0xffffff) + 0x51000000;
  local_20[1] = (local_2c & 0xffffff) + 0x50000000;
  local_20[2] = local_28 << 0x10 | DAT_000012a0 - 6U;
  iVar1 = DAT_000012a8;
  local_20[3] = DAT_000012a4 + 0x10;
  local_20[4] = 0x47000000;
  local_20[5] = ((uint)*(byte *)(local_24 + 9) + iVar1 & 0x7fffff) + 0x20800000;
  local_20[6] = (local_18 & 0xffffff) + 0x32000000;
  local_20[7] = *(uint3 *)(iVar2 + 0x47c) + 0x33000000;
  local_20[8] = *(uint3 *)(iVar2 + 0x47f) + 0x33000000;
  local_20[9] = DAT_000012ac;
  local_20[10] = -0x10000000;
  return;
}



/* ======================================================================
 * 000012b4  hif_mark_confirm_pending
 * ====================================================================== */

void hif_mark_confirm_pending(void)

{
  int iVar1;
  
  iVar1 = pipe_find_by_mac_upper();
  if ((iVar1 != 0) && ((int)((uint)*(byte *)(iVar1 + 6) << 0x1d) < 0)) {
    *(byte *)(iVar1 + 6) = *(byte *)(iVar1 + 6) | 8;
  }
  return;
}



/* ======================================================================
 * 000012cc  mac_set_state3
 * ====================================================================== */

void mac_set_state3(void)

{
  *(undefined2 *)(DAT_000012d4 + 8) = 3;
  return;
}



/* ======================================================================
 * 000012d8  key_lookup_for_frame
 * ====================================================================== */

/* key_lookup_for_frame(if_id, addr, key_sel) -- find the key entry to use for a
   frame.  Returns a pointer to the key slot, or NULL.
   
   Walks the **24-entry** key table at g_keys (stride 0xA4), matching on:
     slot+0x398 == 1        slot in use
     slot+0x39A == if_id    owning vif
     slot+0x399             key type byte
   
   `key_sel == 0xF` means "pairwise, match by MAC": the slot's stored address at
   +0x39C..0x3A1 must equal `addr`, and type bit 0 must be set.
   
   Otherwise a group key is wanted and the low bits of `key_sel` are the key index,
   compared against a type-specific offset within the slot:
   
     type 0 (WEP)        index at slot+0x39C
     type 2 (TKIP)       index at slot+0x3B4
     type 4 / 8 (CCMP)   index at slot+0x3AC
     type 6 (IGTK/BIP)   index at slot+0x3BC, compared against (key_sel & 7) >> 2
   
   The matched slot number is also written to `vif+0x112` as a side effect, which is
   how the TX and RX paths later find the same key without repeating the search.
   
   **24 key slots total, shared across all vifs** -- more than the 4 that WSM
   `ADD_KEY (0x000C... )` style APIs usually expose, and notably more than
   mainline uses.  Type 6 being IGTK is consistent with
   tx_select_key_and_cipher's management-frame-protection path (key types 4 and 5
   there), so **802.11w PMF has key-table support in this firmware**.
   
   Callers: tx_select_key_and_cipher (0x0000E1F0) on TX, rx_decrypt_and_verify
   (0x0000BEBA) and tx_send_template_frame on RX/mgmt. */

char * key_lookup_for_frame(uint param_1,short *param_2,uint param_3)

{
  byte bVar1;
  int iVar2;
  char *pcVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  char *pcVar7;
  bool bVar8;
  uint local_1c;
  
  bVar8 = param_3 == 0xf;
  pcVar7 = (char *)0x0;
  local_1c = param_3;
  if ((int)(param_3 << 0x1b) < 0) {
    local_1c = (param_3 & 7) >> 2;
    param_3 = param_3 & 3;
  }
  iVar4 = param_1 * 0x3b0 + DAT_0000160c;
  uVar6 = 0;
  *(undefined1 *)(iVar4 + 0x112) = 0;
  do {
    iVar2 = uVar6 * 0xa4 + DAT_00001610;
    pcVar3 = (char *)(iVar2 + 0x398);
    if ((*pcVar3 == '\x01') && (*(byte *)(iVar2 + 0x39a) == param_1)) {
      bVar1 = *(byte *)(iVar2 + 0x399);
      if (((bVar1 & 1) != 0) &&
         ((((bVar8 && (*(short *)(iVar2 + 0x39c) == *param_2)) &&
           (*(short *)(iVar2 + 0x39e) == param_2[1])) && (*(short *)(iVar2 + 0x3a0) == param_2[2])))
         ) {
        *(char *)(iVar4 + 0x112) = (char)uVar6;
        return pcVar3;
      }
      if ((bVar1 & 1) == 0 && !bVar8) {
        if (bVar1 == 0) {
          uVar5 = (uint)*(byte *)(iVar2 + 0x39c);
LAB_00001374:
          if (uVar5 != param_3) goto LAB_0000138e;
        }
        else {
          if (bVar1 == 2) {
            uVar5 = (uint)*(byte *)(iVar2 + 0x3b4);
            goto LAB_00001374;
          }
          if ((bVar1 == 4) || (bVar1 == 8)) {
            uVar5 = (uint)*(byte *)(iVar2 + 0x3ac);
            goto LAB_00001374;
          }
          if ((bVar1 != 6) || (*(byte *)(iVar2 + 0x3bc) != local_1c)) goto LAB_0000138e;
        }
        *(char *)(iVar4 + 0x112) = (char)uVar6;
        pcVar7 = pcVar3;
      }
    }
LAB_0000138e:
    uVar6 = uVar6 + 1 & 0xff;
    if (0x17 < uVar6) {
      return pcVar7;
    }
  } while( true );
}



/* ======================================================================
 * 0000139c  lmc_req_find_by_vif
 * ====================================================================== */

undefined4 * lmc_req_find_by_vif(uint param_1,undefined4 *param_2)

{
  char cVar1;
  uint uVar2;
  int iVar3;
  
  uVar2 = 0;
  while (((iVar3 = uVar2 * 0xa4 + DAT_00001610, *(char *)(iVar3 + 0x398) != '\x01' ||
          (*(byte *)(iVar3 + 0x39a) != param_1)) ||
         ((cVar1 = *(char *)(iVar3 + 0x399), cVar1 != '\0' &&
          ((cVar1 != '\x02' && (cVar1 != '\x04'))))))) {
    uVar2 = uVar2 + 1 & 0xff;
    if (0x17 < uVar2) {
      return (undefined4 *)0x0;
    }
  }
  *param_2 = *(undefined4 *)(iVar3 + 0x3ec);
  *(undefined2 *)(param_2 + 1) = *(undefined2 *)(iVar3 + 0x3f0);
  return param_2;
}



/* ======================================================================
 * 000013e4  lmc_msg_dispatch_by_type
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00001406) */
/* WARNING: Removing unreachable block (ram,0x00001406) */

void lmc_msg_dispatch_by_type(int param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  
  *(bool *)param_4 = (*(byte *)(param_1 + 1) & 1) == 0;
  uVar1 = (uint)*(byte *)(param_1 + 1);
                    /* WARNING: Could not recover jumptable at 0x00001406. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0000140a <= uVar1) {
    uVar1 = (uint)DAT_0000140a;
  }
  (*(code *)((uint)*(byte *)(uVar1 + 0x140b) * 2 + 0x140b))(param_2 + 8,param_1 + 0xc);
  return;
}



/* ======================================================================
 * 000015d0  rx_copy_ccmp_hdr_fields
 * ====================================================================== */

void rx_copy_ccmp_hdr_fields(int param_1,ushort *param_2)

{
  *(ushort *)(param_1 + 0x28) = *param_2 & (short)DAT_00001614 + 0x70U;
  *(ushort *)(param_1 + 0x2a) = param_2[2];
  *(ushort *)(param_1 + 0x2c) = param_2[3];
  *(ushort *)(param_1 + 0x2e) = param_2[4];
  *(ushort *)(param_1 + 0x30) = param_2[5];
  *(ushort *)(param_1 + 0x32) = param_2[6];
  *(ushort *)(param_1 + 0x34) = param_2[7];
  *(ushort *)(param_1 + 0x36) = param_2[8];
  *(ushort *)(param_1 + 0x38) = param_2[9];
  *(ushort *)(param_1 + 0x3a) = param_2[10];
  fw_memcpy((void *)(param_1 + 0x3c),param_2 + 0xc,0xc);
  return;
}



/* ======================================================================
 * 00001694  mic_build_aad_rx
 * ====================================================================== */

void mic_build_aad_rx(int *param_1)

{
  ushort uVar1;
  undefined4 uVar2;
  ushort uVar3;
  uint uVar4;
  void *src;
  void *pvVar5;
  int iVar6;
  void *dst;
  uint uVar7;
  int iVar8;
  undefined1 *src_00;
  byte bVar9;
  
  src = (void *)param_1[7];
  bVar9 = 0;
  iVar6 = *(int *)(*param_1 + 0x10);
  dst = (void *)((int)src + iVar6 + -8);
  fw_memcpy((void *)(*param_1 + 0xa8),src,iVar6 + 8);
  iVar8 = *param_1;
  uVar1 = *(ushort *)(iVar8 + 0xa8);
  uVar3 = uVar1 & 0x300;
  if ((uVar1 & 0x300) == 0) {
    pvVar5 = (void *)(iVar8 + 0xac);
LAB_000016ec:
    fw_memcpy(dst,pvVar5,6);
    pvVar5 = (void *)(iVar8 + 0xb2);
  }
  else {
    if (uVar3 == 0x100) {
      pvVar5 = (void *)(iVar8 + 0xb8);
      goto LAB_000016ec;
    }
    if (uVar3 == 0x200) {
      fw_memcpy(dst,(void *)(iVar8 + 0xac),6);
      pvVar5 = (void *)(iVar8 + 0xb8);
    }
    else {
      if (uVar3 != 0x300) goto LAB_000016fe;
      fw_memcpy(dst,(void *)(iVar8 + 0xb8),6);
      pvVar5 = (void *)(iVar8 + 0xc0);
    }
  }
  fw_memcpy((void *)((int)src + iVar6 + -2),pvVar5,6);
LAB_000016fe:
  uVar4 = (uint)*(ushort *)(iVar8 + 0xa8);
  if ((uVar4 & 0x8f) == 0x88) {
    if ((uVar4 & 0x3ff) >> 8 == 3) {
      bVar9 = (byte)*(undefined2 *)(iVar8 + 0xc6);
    }
    else {
      bVar9 = (byte)*(undefined2 *)(iVar8 + 0xc0);
    }
    bVar9 = bVar9 & 0xf;
  }
  *(byte *)((int)src + iVar6 + 4) = bVar9;
  *(undefined1 *)((int)src + iVar6 + 5) = 0;
  *(undefined1 *)((int)src + iVar6 + 6) = 0;
  *(undefined1 *)((int)src + iVar6 + 7) = 0;
  *(void **)(*param_1 + 0x90) = dst;
  iVar6 = param_1[6] - *(int *)(*param_1 + 0x10);
  uVar7 = iVar6 + 4U >> 2;
  src_00 = (undefined1 *)(param_1[7] + param_1[6] + -0xc);
  fw_memcpy((void *)(*param_1 + 0xd0),src_00,8);
  *src_00 = 0x5a;
  for (uVar4 = 0; uVar2 = DAT_00001880, src_00 = src_00 + 1,
      uVar4 < (uVar7 * 4 - (iVar6 + -3) & 0xff); uVar4 = uVar4 + 1 & 0xff) {
    *src_00 = 0;
  }
  *(uint *)(*param_1 + 0x94) = uVar7;
  *(undefined4 *)(*param_1 + 0xa0) = uVar2;
  mic_submit_or_queue(*param_1 + 0x8c);
  return;
}



/* ======================================================================
 * 00001794  rx_reorder_or_deliver
 * ====================================================================== */

void rx_reorder_or_deliver(int *param_1)

{
  char cVar1;
  ushort uVar2;
  undefined4 uVar3;
  ushort uVar4;
  int iVar5;
  uint uVar6;
  void *src;
  void *pvVar7;
  int *piVar8;
  void *dst;
  uint uVar9;
  int iVar10;
  undefined1 *src_00;
  byte bVar11;
  
  cVar1 = *(char *)(*param_1 + 9);
  if ((cVar1 != '\x02') && (cVar1 != '\x03')) {
    piVar8 = *(int **)*param_1;
    iVar5 = *piVar8;
    if ((*(char *)(iVar5 + 9) == '\x02') || (*(char *)(iVar5 + 9) == '\x03')) {
      fw_memcpy((void *)piVar8[7],(void *)(iVar5 + 0xa8),*(int *)(iVar5 + 0x10) + 8);
      fw_memcpy((void *)(piVar8[7] + piVar8[6] + -0xc),(void *)(*piVar8 + 0xd0),8);
      iVar5 = *piVar8;
      if ((*(int *)(iVar5 + 0x98) != *(int *)(iVar5 + 0xd0)) ||
         (*(int *)(iVar5 + 0x9c) != *(int *)(iVar5 + 0xd4))) {
        iVar5 = (uint)*(byte *)((int)piVar8 + 0x2a) * 0x3b0 + DAT_0000187c;
        *(int *)(iVar5 + 0x6c) = *(int *)(iVar5 + 0x6c) + 1;
        piVar8[2] = 0x13;
        piVar8[6] = *(int *)(*piVar8 + 0x10) + 8;
        rx_indication_build_and_send(piVar8);
        return;
      }
    }
    rx_deliver_or_queue_mgmt(piVar8);
    return;
  }
  src = (void *)param_1[7];
  bVar11 = 0;
  iVar5 = *(int *)(*param_1 + 0x10);
  dst = (void *)((int)src + iVar5 + -8);
  fw_memcpy((void *)(*param_1 + 0xa8),src,iVar5 + 8);
  iVar10 = *param_1;
  uVar2 = *(ushort *)(iVar10 + 0xa8);
  uVar4 = uVar2 & 0x300;
  if ((uVar2 & 0x300) == 0) {
    pvVar7 = (void *)(iVar10 + 0xac);
LAB_000016ec:
    fw_memcpy(dst,pvVar7,6);
    pvVar7 = (void *)(iVar10 + 0xb2);
  }
  else {
    if (uVar4 == 0x100) {
      pvVar7 = (void *)(iVar10 + 0xb8);
      goto LAB_000016ec;
    }
    if (uVar4 == 0x200) {
      fw_memcpy(dst,(void *)(iVar10 + 0xac),6);
      pvVar7 = (void *)(iVar10 + 0xb8);
    }
    else {
      if (uVar4 != 0x300) goto LAB_000016fe;
      fw_memcpy(dst,(void *)(iVar10 + 0xb8),6);
      pvVar7 = (void *)(iVar10 + 0xc0);
    }
  }
  fw_memcpy((void *)((int)src + iVar5 + -2),pvVar7,6);
LAB_000016fe:
  uVar6 = (uint)*(ushort *)(iVar10 + 0xa8);
  if ((uVar6 & 0x8f) == 0x88) {
    if ((uVar6 & 0x3ff) >> 8 == 3) {
      bVar11 = (byte)*(undefined2 *)(iVar10 + 0xc6);
    }
    else {
      bVar11 = (byte)*(undefined2 *)(iVar10 + 0xc0);
    }
    bVar11 = bVar11 & 0xf;
  }
  *(byte *)((int)src + iVar5 + 4) = bVar11;
  *(undefined1 *)((int)src + iVar5 + 5) = 0;
  *(undefined1 *)((int)src + iVar5 + 6) = 0;
  *(undefined1 *)((int)src + iVar5 + 7) = 0;
  *(void **)(*param_1 + 0x90) = dst;
  iVar5 = param_1[6] - *(int *)(*param_1 + 0x10);
  uVar9 = iVar5 + 4U >> 2;
  src_00 = (undefined1 *)(param_1[7] + param_1[6] + -0xc);
  fw_memcpy((void *)(*param_1 + 0xd0),src_00,8);
  *src_00 = 0x5a;
  for (uVar6 = 0; uVar3 = DAT_00001880, src_00 = src_00 + 1,
      uVar6 < (uVar9 * 4 - (iVar5 + -3) & 0xff); uVar6 = uVar6 + 1 & 0xff) {
    *src_00 = 0;
  }
  *(uint *)(*param_1 + 0x94) = uVar9;
  *(undefined4 *)(*param_1 + 0xa0) = uVar3;
  mic_submit_or_queue(*param_1 + 0x8c);
  return;
}



/* ======================================================================
 * 000017a8  rx_check_pn_replay
 * ====================================================================== */

undefined4 rx_check_pn_replay(int *param_1)

{
  int iVar1;
  uint uVar2;
  uint *puVar3;
  int iVar4;
  int iVar5;
  
  uVar2 = param_1[4] & 7;
  if ((uVar2 == 3) || (uVar2 == 2)) {
    iVar4 = *param_1;
    puVar3 = (uint *)(*(int *)(iVar4 + 0x18) + 0x2c);
    if ((*(byte *)((int)param_1 + 0x2b) & 0xf) != 0) {
      puVar3 = (uint *)((uint)*(byte *)((int)param_1 + 0x22) * 8 + *(int *)(iVar4 + 0x18) + 0x34);
    }
    uVar2 = *(uint *)(iVar4 + 0x24);
    if (((*puVar3 < uVar2) ||
        ((uVar2 == *puVar3 && ((ushort)puVar3[1] < *(ushort *)(iVar4 + 0x28))))) &&
       ((*(short *)(iVar4 + 0x28) != 0 || (uVar2 != 0)))) {
      *(short *)(puVar3 + 1) = *(short *)(iVar4 + 0x28);
      *puVar3 = *(uint *)(*param_1 + 0x24);
      return 1;
    }
    if ((*(byte *)((int)param_1 + 0x2b) & 0xf) != 0) {
      return 0;
    }
    iVar4 = (uint)*(byte *)((int)param_1 + 0x2a) * 0x3b0 + DAT_0000187c;
    *(int *)(iVar4 + 0x9c) = *(int *)(iVar4 + 0x9c) + 1;
    return 0;
  }
  if (uVar2 != 4) {
    return 1;
  }
  iVar4 = *param_1;
  iVar5 = (uint)*(byte *)((int)param_1 + 0x22) * 8 + *(int *)(iVar4 + 0x18);
  uVar2 = *(uint *)(iVar4 + 0x28);
  if (uVar2 <= *(uint *)(iVar5 + 0x38)) {
    if (uVar2 != *(uint *)(iVar5 + 0x38)) {
      return 0;
    }
    if (*(uint *)(iVar4 + 0x24) <= *(uint *)(iVar5 + 0x34)) {
      return 0;
    }
  }
  if ((uVar2 == 0) && (*(int *)(iVar4 + 0x24) == 0)) {
    return 0;
  }
  iVar1 = *(int *)((uint)*(byte *)((int)param_1 + 0x2a) * 0x3b0 + DAT_0000187c + 0x1c);
  if (iVar1 << 0x1d < 0) {
    if ((*(uint *)(iVar4 + 0x24) & 1) == 0) goto LAB_0000186a;
  }
  else if ((*(uint *)(iVar4 + 0x24) & 1) != 0) goto LAB_0000186a;
  if ((-1 < param_1[4] << 0xc) && (-1 < iVar1 << 0x1c)) {
    return 0;
  }
LAB_0000186a:
  *(uint *)(iVar5 + 0x34) = *(uint *)(iVar4 + 0x24);
  *(undefined4 *)(iVar5 + 0x38) = *(undefined4 *)(*param_1 + 0x28);
  return 1;
}



/* ======================================================================
 * 00001884  tx_pending_count
 * ====================================================================== */

int tx_pending_count(void)

{
  return (uint)*(byte *)(DAT_000018d4 + 4) + (uint)*(byte *)(DAT_000018d4 + 5);
}



/* ======================================================================
 * 0000188e  tx_abort_frames_for_vif
 * ====================================================================== */

/* tx_abort_frames_for_vif(if_id) -- mark this vif's in-flight TX frames aborted
   with result 0x14.  Returns 1 if anything was aborted.
   
   *** FIRMWARE BUG: THE LOOPS NEVER ADVANCE THE POINTER. ***
   From the disassembly (the decompiler renders this faithfully, it is not a lifting
   artifact):
   
     ldr  r2,[0x000018D8]      ; pool A base
     add  r2,#0x60             ; r2 fixed for the whole loop
   loopA:
     ldrh r6,[r2,#0x10]        ; always descriptor 0's result field
     ...
     add  r1,r1,#0x1
     cmp  r1,#0x3
     bcc  loopA                ; r1 counts, but r2 is NEVER incremented
   
     ldr  r2,[0x000018DC]      ; pool B base
     add  r2,#0x60
   loopB:  ... cmp r1,#0x1E ; bcc loopB      ; same defect
   
   `r2` is loaded once per loop and never advanced, and `r1` is only ever compared
   against the bound -- it is not used in any address computation.  So each loop
   re-tests **descriptor 0 of its pool** 3 and 30 times respectively.
   
   Consequence: when a vif is removed or reset, only the FIRST descriptor of each
   pool is aborted.  Frames sitting in descriptors 1..N-1 that belong to the
   departing vif keep their old result and are not force-completed.  Suspect this
   first if interface teardown / `WSM_RESET` leaves stale TX credits outstanding or
   a frame completes against the wrong vif after a mode change.  Not exploitable
   from the host, and it self-clears once those frames complete normally.
   
   *** USEFUL SIDE EFFECT: THE LOOP BOUNDS REVEAL BOTH POOL DEPTHS. ***
   The author's intent was to walk each pool completely, so:
     pool A (0x000018D8) : **3**  descriptors  -- matches tx_ctx_pool_init exactly
     pool B (0x000018DC) : **30** descriptors  -- matches input_buffers = 30
   
   That independently answers the previously open question of how deep the host TX
   descriptor pool is (tx_wsm_buf_alloc, head 0x040087B0): thirty, one per host
   input buffer.  Both figures now have two independent sources.
   
   Offsets used are pool_base + 0x60 + {0xBD-0x60 = if_id, 0x10 = +0x70 result}.
   Result 0x14 is the same status tx_lmac_req_submit returns for an unmapped link. */

undefined4 tx_abort_frames_for_vif(uint param_1)

{
  byte bVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  uint uVar5;
  
  iVar2 = DAT_000018d8;
  bVar1 = *(byte *)(DAT_000018d8 + 0xbd);
  uVar4 = 0;
  uVar5 = 0;
  do {
    if ((bVar1 == param_1) && (*(short *)(iVar2 + 0x70) != 0xff)) {
      uVar4 = 1;
      *(undefined2 *)(iVar2 + 0x70) = 0x14;
    }
    iVar3 = DAT_000018dc;
    uVar5 = uVar5 + 1;
  } while (uVar5 < 3);
  bVar1 = *(byte *)(DAT_000018dc + 0xbd);
  uVar5 = 0;
  do {
    if ((bVar1 == param_1) && (*(short *)(iVar3 + 0x70) != 0xff)) {
      uVar4 = 1;
      *(undefined2 *)(iVar3 + 0x70) = 0x14;
    }
    uVar5 = uVar5 + 1;
  } while (uVar5 < 0x1e);
  return uVar4;
}



/* ======================================================================
 * 000018e0  ps_set_mode_and_notify_ap
 * ====================================================================== */

void ps_set_mode_and_notify_ap(int param_1,int param_2)

{
  int iVar1;
  
  iVar1 = param_1 * 0x104 + DAT_00001b3c;
  if (param_2 == 1) {
    mac_set_ps_bit(param_1);
    *(undefined1 *)(iVar1 + 0x40) = 3;
  }
  else {
    *(undefined4 *)(iVar1 + 0x54) = 0;
    *(ushort *)(iVar1 + 0x44) = *(ushort *)(iVar1 + 0x44) & 0xfeff;
    *(undefined1 *)(iVar1 + 0x40) = 4;
    mac_set_ps_bit(param_1,0);
  }
  tx_send_null_data(param_1,4);
  return;
}



/* ======================================================================
 * 00001924  ps_clear_ind_counters
 * ====================================================================== */

void ps_clear_ind_counters(uint param_1,int param_2)

{
  int iVar1;
  
  iVar1 = param_1 * 0x104 + DAT_00001b3c;
  if (param_1 < 2) {
    *(undefined2 *)(iVar1 + 0x13a) = 0;
  }
  if ((param_2 == 0) && (param_1 < 2)) {
    *(undefined2 *)(iVar1 + 0x13a) = 0;
    *(ushort *)(iVar1 + 0x44) = *(ushort *)(iVar1 + 0x44) & 0xffdf | 4;
    timer_start(iVar1 + 0xac,DAT_00001b40);
  }
  return;
}



/* ======================================================================
 * 00001960  ps_on_beacon_rx
 * ====================================================================== */

/* ps_on_beacon_rx(if_id, tim_result) -- the station power-save decision taken on
   every received beacon.  `tim_result` is rx_beacon_check_tim_for_us's packed
   return: high byte = group-addressed (DTIM) pending, low byte = our AID's bit set.
   
     if (tim_result >> 8)   set flag 0x20 in ps+0x44, bump ps+0x13A (mcast pending)
     else                   clear 0x20, zero ps+0x13A
     if (tim_result & 0xFF)  set flag 0x10, bump ps+0xFD and a global counter
     else                    clear 0x10, zero ps+0xFD
   
   Then, when the vif is in PS (ps+0x40 == 1) and unicast traffic is pending, it
   either sends a **QoS-null** (when the UAPSD mask at ps+0x5A selects it) or a
   **PS-Poll**, or defers with a timer; when nothing is pending it lets
   ps_try_enter_sleep_all take the radio down.
   
   *** DYNAMIC LISTEN INTERVAL -- a firmware behaviour with no WSM control. ***
   The tail implements an adaptive listen interval using per-vif fields:
   
     vif+0x378  current listen interval (beacons)
     vif+0x37A  maximum allowed (0 disables the whole mechanism)
     vif+0x37C  floor to fall back to
     vif+0x37E  consecutive-idle beacon counter
   
     if (no traffic pending && no QoS-null was sent) {
         if (++vif[0x37E] == 4) {              /* four idle beacons */
             vif[0x378] += 2;                  /* grow by 2 */
             if (vif[0x378] > vif[0x37A]) vif[0x378] = vif[0x37A];
             ps[0x12C] = vif->beacon_interval * vif[0x378];
         }
     } else {
         vif[0x378] = vif[0x37C];              /* any traffic snaps back to floor */
     }
     vif[0x37E] = 0;
   
   So the firmware **lengthens its own wake interval after four consecutive idle
   beacons and collapses it the moment traffic appears.**  The host never sees this:
   the listen interval it programmed via SET_PM_MODE is only the floor
   (vif+0x37C) and ceiling (vif+0x37A).
   
   Relevance to throughput: this only engages while idle, and the first frame in
   either direction resets it to the floor, so it should not affect a saturated
   transfer.  Worth knowing for latency-sensitive tests though -- an idle link can be
   sleeping up to `max` beacons deep when traffic resumes, which shows up as a
   first-packet delay rather than a throughput loss. */

void ps_on_beacon_rx(int param_1,uint param_2)

{
  ushort uVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  ushort *puVar5;
  short sVar6;
  int iVar7;
  int iVar8;
  int iVar9;
  
  iVar7 = param_1 * 0x3b0 + DAT_00001b44;
  iVar8 = param_1 * 0x104 + DAT_00001b3c;
  iVar3 = param_1 * 0x104 + DAT_00001b3c;
  if (param_2 >> 8 == 0) {
    *(ushort *)(iVar8 + 0x44) = *(ushort *)(iVar3 + 0x44) & 0xffdf;
    *(undefined2 *)(iVar3 + 0x13a) = 0;
  }
  else {
    *(ushort *)(iVar8 + 0x44) = *(ushort *)(iVar3 + 0x44) | 0x20;
    *(short *)(iVar3 + 0x13a) = *(short *)(iVar3 + 0x13a) + 1;
  }
  iVar9 = param_1 * 0x104 + DAT_00001b3c;
  if ((param_2 & 0xff) == 0) {
    *(ushort *)(iVar8 + 0x44) = *(ushort *)(iVar8 + 0x44) & 0xffef;
    *(undefined1 *)(iVar9 + 0xfd) = 0;
  }
  else {
    *(ushort *)(iVar8 + 0x44) = *(ushort *)(iVar8 + 0x44) | 0x10;
    iVar2 = DAT_00001b48;
    *(char *)(iVar9 + 0xfd) = *(char *)(iVar9 + 0xfd) + '\x01';
    *(int *)(iVar2 + 0x20) = *(int *)(iVar2 + 0x20) + 1;
  }
  iVar2 = DAT_00001b3c;
  *(undefined1 *)(iVar8 + 0x115) = 0;
  if (*(char *)(iVar2 + 0x2a) == '\x02') {
    *(undefined1 *)(iVar2 + 0x2a) = 3;
  }
  if (*(char *)(iVar8 + 0x40) == '\0') {
    if ((((int)((uint)*(ushort *)(iVar8 + 0x44) << 0x1b) < 0) || (*(short *)(iVar3 + 0x138) != 0))
       && (*(char *)(iVar8 + 0x41) != '\0')) {
      mac_set_ps_bit(param_1,0);
      tx_send_null_data(param_1,4);
    }
    if (2 < *(byte *)(iVar2 + 0x2a)) {
      ps_try_enter_sleep_all();
    }
    if (((*(char *)(iVar9 + 0xfc) != '\0') && (-1 < (int)((uint)*(ushort *)(iVar8 + 0x44) << 0x1c)))
       && (-1 < (int)((uint)*(ushort *)(iVar8 + 0x44) << 0x1b))) {
      ps_reevaluate_all();
    }
  }
  else if (*(char *)(iVar8 + 0x40) == '\x01') {
    *(undefined1 *)(iVar9 + 0xfe) = 0;
    uVar1 = *(ushort *)(iVar8 + 0x44);
    if ((int)((uint)uVar1 << 0x1b) < 0) {
      if (*DAT_00001b4c << 0x18 < 0) {
        *(ushort *)(iVar8 + 0x44) = uVar1 & 0xffef;
        *(ushort *)(iVar8 + 0x46) = *(ushort *)(iVar8 + 0x46) | 0x10;
        ps_try_enter_sleep_all();
      }
      else {
        if (*(char *)(iVar9 + 0xfc) != '\0') {
          *(ushort *)(iVar8 + 0x44) = uVar1 | 8;
          timer_start(iVar8 + 0xe8,*(undefined4 *)(iVar8 + 0x118));
          if (*(byte *)(iVar2 + 0x2a) < 2) {
            uVar4 = fw_read_timer();
            *(undefined4 *)(iVar8 + 0x140) = uVar4;
            ps_set_mode_and_notify_ap(param_1,0);
            return;
          }
        }
        if ((*(ushort *)(iVar8 + 0x5a) & 0xf10) == 0xf00) {
          *(ushort *)(iVar8 + 0x44) = *(ushort *)(iVar8 + 0x44) & 0xffef;
          *(undefined1 *)(iVar9 + 0xfe) = 1;
          tx_send_qos_null(param_1,1);
        }
        else {
          *(undefined1 *)(iVar8 + 0x114) = 0;
          tx_send_ps_poll(param_1);
        }
      }
    }
    else if (*(short *)(iVar3 + 0x138) != 0) {
      mac_set_ps_bit(param_1,1);
      tx_send_null_data(param_1,4);
    }
    puVar5 = (ushort *)(iVar7 + 0x378);
    if (*(short *)(iVar7 + 0x37a) != 0) {
      if (((*(ushort *)(iVar8 + 0x44) & 0x7f) >> 4 == 0) && (*(char *)(iVar9 + 0xfe) == '\0')) {
        sVar6 = *(short *)(iVar7 + 0x37e) + 1;
        *(short *)(iVar7 + 0x37e) = sVar6;
        if (sVar6 != 4) goto LAB_00001b32;
        uVar1 = *puVar5;
        *puVar5 = uVar1 + 2;
        if (*(ushort *)(iVar7 + 0x37a) < (ushort)(uVar1 + 2)) {
          *puVar5 = *(ushort *)(iVar7 + 0x37a);
        }
        *(uint *)(iVar8 + 300) = *(int *)(iVar7 + 0x118) * (uint)*puVar5;
      }
      else {
        *puVar5 = *(ushort *)(iVar7 + 0x37c);
      }
      *(undefined2 *)(iVar7 + 0x37e) = 0;
    }
  }
LAB_00001b32:
  ps_schedule_next_tbtt_wake(param_1);
  return;
}



/* ======================================================================
 * 00001b50  qos_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00001b54) */
/* WARNING: Removing unreachable block (ram,0x00001b54) */

void qos_dispatch(uint param_1)

{
                    /* WARNING: Could not recover jumptable at 0x00001b54. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_00001b58 <= param_1) {
    param_1 = (uint)DAT_00001b58;
  }
  (*(code *)((uint)*(byte *)(param_1 + 0x1b59) * 2 + 0x1b59))();
  return;
}



/* ======================================================================
 * 00001b70  qos_ac_to_param
 * ====================================================================== */

undefined4 qos_ac_to_param(int param_1)

{
  if (param_1 == 0) {
    return 2;
  }
  if (param_1 == 1) {
    return 3;
  }
  if (param_1 == 2) {
    return 5;
  }
  if (param_1 == 3) {
    return 7;
  }
  fw_assert(s_qos_c_00001ba0,0xcf,0x13);
  return 1;
}



/* ======================================================================
 * 00001ba8  tkip_key_mix
 * ====================================================================== */

void tkip_key_mix(uint *param_1,ushort *param_2,ushort *param_3,uint param_4)

{
  uint uVar1;
  uint uVar2;
  uint uVar3;
  uint uVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  
  uVar1 = param_3[4] + param_4 & 0xffff;
  uVar2 = *param_2 ^ uVar1;
  uVar4 = (uint)(*(ushort *)(g_tkip_sbox_lo + (uVar2 & 0xff) * 2) ^
                *(ushort *)(g_tkip_sbox_hi + (uVar2 >> 8) * 2)) + (uint)*param_3 & 0xffff;
  uVar2 = param_2[1] ^ uVar4;
  uVar5 = (uint)(*(ushort *)(g_tkip_sbox_hi + (uVar2 >> 8) * 2) ^
                *(ushort *)(g_tkip_sbox_lo + (uVar2 & 0xff) * 2)) + (uint)param_3[1] & 0xffff;
  uVar2 = param_2[2] ^ uVar5;
  uVar6 = (uint)(*(ushort *)(g_tkip_sbox_hi + (uVar2 >> 8) * 2) ^
                *(ushort *)(g_tkip_sbox_lo + (uVar2 & 0xff) * 2)) + (uint)param_3[2] & 0xffff;
  uVar2 = param_2[3] ^ uVar6;
  uVar7 = (uint)(*(ushort *)(g_tkip_sbox_hi + (uVar2 >> 8) * 2) ^
                *(ushort *)(g_tkip_sbox_lo + (uVar2 & 0xff) * 2)) + (uint)param_3[3] & 0xffff;
  uVar2 = param_2[4] ^ uVar7;
  uVar3 = (uint)(*(ushort *)(g_tkip_sbox_hi + (uVar2 >> 8) * 2) ^
                *(ushort *)(g_tkip_sbox_lo + (uVar2 & 0xff) * 2)) + (uint)param_3[4] & 0xffff;
  uVar2 = param_2[5] ^ uVar3;
  uVar1 = (*(ushort *)(g_tkip_sbox_lo + (uVar2 & 0xff) * 2) ^
          *(ushort *)(g_tkip_sbox_hi + (uVar2 >> 8) * 2)) + uVar1;
  uVar2 = param_2[6] ^ uVar1;
  uVar4 = (uVar2 << 0xf ^ (uVar2 & 0xffff) >> 1) + uVar4 & 0xffff;
  uVar2 = param_2[7] ^ uVar4;
  uVar5 = (uVar2 << 0xf ^ uVar2 >> 1) + uVar5;
  uVar2 = uVar5 & 0xffff;
  uVar2 = (uVar2 >> 1 ^ uVar2 << 0xf) + uVar6 & 0xffff;
  uVar7 = (uVar2 >> 1 ^ uVar2 << 0xf) + uVar7;
  uVar6 = uVar7 & 0xffff;
  uVar3 = (uVar6 >> 1 ^ uVar6 << 0xf) + uVar3 & 0xffff;
  uVar1 = (uVar3 >> 1 ^ uVar3 << 0xf) + uVar1;
  *param_1 = (((*param_2 ^ uVar1) & 0x1ff) >> 1) << 0x18 |
             (param_4 & 0xff) << 0x10 | (param_4 & 0xffffff00 | param_4 >> 8) & 0x7fff | 0x2000;
  param_1[1] = uVar5 * 0x10000 | uVar4;
  param_1[2] = uVar7 * 0x10000 | uVar2;
  param_1[3] = uVar1 * 0x10000 | uVar3;
  return;
}



/* ======================================================================
 * 00001d3c  tkip_phase1_mix
 * ====================================================================== */

void tkip_phase1_mix(undefined2 *param_1,int param_2,ushort *param_3,uint param_4)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  uint uVar8;
  uint uVar9;
  uint uVar10;
  uint uVar11;
  uint local_20;
  uint local_1c;
  
  local_1c = param_4 & 0xffff;
  local_20 = param_4 >> 0x10;
  uVar5 = (uint)*param_3;
  uVar7 = (uint)param_3[1];
  uVar4 = (uint)param_3[2];
  uVar3 = 0;
  do {
    iVar1 = (uVar3 & 1) * 2;
    iVar2 = iVar1 + param_2;
    uVar10 = (uint)CONCAT11(*(undefined1 *)(iVar2 + 1),*(undefined1 *)(param_2 + iVar1));
    uVar11 = uVar10 ^ uVar4;
    uVar8 = (*(ushort *)(g_tkip_sbox_lo + (uVar11 & 0xff) * 2) ^
            *(ushort *)(g_tkip_sbox_hi + (uVar11 >> 8) * 2)) + local_1c;
    local_1c = uVar8 & 0xffff;
    uVar11 = *(ushort *)(iVar2 + 4) ^ local_1c;
    uVar9 = (*(ushort *)(g_tkip_sbox_lo + (uVar11 & 0xff) * 2) ^
            *(ushort *)(g_tkip_sbox_hi + (uVar11 >> 8) * 2)) + local_20;
    local_20 = uVar9 & 0xffff;
    uVar11 = *(ushort *)(iVar2 + 8) ^ local_20;
    uVar6 = (*(ushort *)(g_tkip_sbox_hi + (uVar11 >> 8) * 2) ^
            *(ushort *)(g_tkip_sbox_lo + (uVar11 & 0xff) * 2)) + uVar5;
    uVar5 = uVar6 & 0xffff;
    uVar11 = *(ushort *)(iVar2 + 0xc) ^ uVar5;
    uVar11 = (*(ushort *)(g_tkip_sbox_hi + (uVar11 >> 8) * 2) ^
             *(ushort *)(g_tkip_sbox_lo + (uVar11 & 0xff) * 2)) + uVar7;
    uVar7 = uVar11 & 0xffff;
    uVar10 = uVar10 ^ uVar7;
    uVar10 = (*(ushort *)(g_tkip_sbox_hi + (uVar10 >> 8) * 2) ^
             *(ushort *)(g_tkip_sbox_lo + (uVar10 & 0xff) * 2)) + uVar4 + uVar3;
    uVar4 = uVar10 & 0xffff;
    uVar3 = uVar3 + 1;
  } while (uVar3 < 8);
  *param_1 = (short)uVar8;
  param_1[1] = (short)uVar9;
  param_1[2] = (short)uVar6;
  param_1[3] = (short)uVar11;
  param_1[4] = (short)uVar10;
  return;
}



/* ======================================================================
 * 00001e3c  tkip_compute_rc4_key
 * ====================================================================== */

undefined8
tkip_compute_rc4_key
          (undefined4 param_1,undefined4 param_2,int *param_3,undefined4 param_4,int *param_5,
          undefined4 param_6)

{
  int iVar1;
  
  iVar1 = *param_5;
  if (*param_3 != iVar1) {
    tkip_phase1_mix(param_4,param_2,param_1,iVar1,param_1,param_2,param_3);
    *param_3 = iVar1;
  }
  tkip_key_mix(param_6,param_2,param_4,(short)param_5[1]);
  return CONCAT44(param_2,param_1);
}



/* ======================================================================
 * 000020e8  bab_originate_addba
 * ====================================================================== */

void bab_originate_addba(int *param_1)

{
  uint uVar1;
  int iVar2;
  undefined1 *puVar3;
  
  uVar1 = (uint)*(byte *)((int)param_1 + 0x69);
  if (uVar1 < 2) {
    if (((uint)*(ushort *)(uVar1 * 2 + DAT_000021b8 + 0xd8) &
        1 << (uint)*(byte *)((int)param_1 + 0x52)) != 0) {
      uVar1 = txpipe_find_by_mac_tid(uVar1,(uint)*(byte *)((int)param_1 + 0x52),*param_1 + 4);
      if (7 < uVar1) {
        uVar1 = link_tbl_add_by_mac(*(undefined1 *)((int)param_1 + 0x69),
                                    *(undefined1 *)((int)param_1 + 0x52),*param_1 + 4);
      }
      *(char *)(param_1 + 0x1b) = (char)uVar1;
      if (uVar1 < 8) {
        iVar2 = uVar1 * 0x38 + DAT_000021bc;
        if ((*(char *)(iVar2 + 0x658) == '\x01') &&
           (puVar3 = (undefined1 *)lmc_msg_alloc(), puVar3 != (undefined1 *)0x0)) {
          puVar3[0x28] = *(undefined1 *)((int)param_1 + 0x69);
          *puVar3 = 5;
          puVar3[4] = *(undefined1 *)((int)param_1 + 0x52);
          puVar3[0x29] = (char)uVar1;
          *(ushort *)(DAT_000021c0 + 0x18) =
               *(ushort *)(DAT_000021c0 + 0x18) & ~(ushort)(1 << (uVar1 & 0xff));
          *(ushort *)(puVar3 + 6) = *(short *)(*param_1 + 0x16) + 0x10U & 0xfff0;
          *(undefined2 *)(puVar3 + 8) = *(undefined2 *)(*param_1 + 4);
          *(undefined2 *)(puVar3 + 10) = *(undefined2 *)(*param_1 + 6);
          *(undefined2 *)(puVar3 + 0xc) = *(undefined2 *)(*param_1 + 8);
          *(char *)(iVar2 + 0x658) = '\x02';
          *(ushort *)(iVar2 + 0x65e) = *(ushort *)(puVar3 + 6) >> 4;
          *(undefined4 *)(iVar2 + 0x660) = 0;
          *(undefined4 *)(iVar2 + 0x664) = 0;
          *(undefined4 *)(iVar2 + 0x668) = 0;
          *(undefined4 *)(iVar2 + 0x66c) = 0;
          *(ushort *)(iVar2 + 0x65a) = *(ushort *)(puVar3 + 6) >> 4;
          evt_flags_set(DAT_000021c4,0x400000);
        }
      }
    }
  }
  return;
}



/* ======================================================================
 * 000021c8  tx_arm_timeout
 * ====================================================================== */

void tx_arm_timeout(int param_1,undefined4 param_2)

{
  undefined4 *puVar1;
  
  puVar1 = DAT_00002464;
  *(undefined4 *)(param_1 + 0x8c) = param_2;
  *puVar1 = param_2;
  *(undefined1 *)((int)puVar1 + -2) = *(undefined1 *)(param_1 + 2);
  *(undefined4 *)(param_1 + 0x90) = 0;
  *(undefined1 *)((int)puVar1 + -3) = 2;
  hw_dma_kick(*(undefined1 *)(param_1 + 2),param_2,*(undefined1 *)(param_1 + 0xc),4);
  return;
}



/* ======================================================================
 * 000021ee  lmc_sched_request_radio
 * ====================================================================== */

undefined4
lmc_sched_request_radio(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  iVar1 = DAT_00002464;
  iVar3 = *(int *)(DAT_00002464 + -0x78);
  if (param_1 == iVar3) {
    return 0;
  }
  *(undefined4 *)(param_1 + 4) = 0;
  if (iVar3 == 0) {
    *(int *)(iVar1 + -0x78) = param_1;
    *(undefined1 *)(param_1 + 0x22) = 3;
    if ((int)((uint)*(byte *)(param_1 + 0xc) << 0x1a) < 0) {
      mac_program_channel_for_vifs_ex
                (*(undefined2 *)(param_1 + 0xe),*(undefined1 *)(param_1 + 0xd),param_3,param_4,
                 param_4);
      *(undefined1 *)(param_1 + 0xc) = 0x30;
    }
    else if (*(short *)(param_1 + 0xe) != *(short *)(DAT_00002468 + 2)) {
      mac_program_channel_for_vifs_ex
                (*(short *)(param_1 + 0xe),*(undefined1 *)(param_1 + 0xd),param_3,param_4,param_4);
    }
  }
  else {
    if (*(short *)(param_1 + 0xe) != *(short *)(iVar3 + 0xe)) {
      *(undefined1 *)(param_1 + 0x22) = 1;
      iVar2 = *(int *)(iVar1 + -0x74);
      if ((iVar2 == 0) || (iVar2 == param_1)) {
        *(int *)(iVar1 + -0x74) = param_1;
      }
      else {
        *(int *)(iVar2 + 4) = param_1;
      }
      if ((*(char *)(iVar3 + 0xd) != '\x02') || (*DAT_0000246c == '\0')) {
        lmc_sched_radio_release(*(undefined4 *)(iVar1 + -0x78));
      }
      return 3;
    }
    if ((int)((uint)*(byte *)(param_1 + 0xc) << 0x1a) < 0) {
      mac_program_channel_for_vifs_ex
                (*(short *)(param_1 + 0xe),*(undefined1 *)(param_1 + 0xd),param_3,param_4,param_4);
      *(undefined1 *)(param_1 + 0xc) = 0x30;
    }
    if (*(byte *)(iVar3 + 0xc) <= *(byte *)(param_1 + 0xc)) {
      *(undefined1 *)(param_1 + 0x22) = 2;
      return 0;
    }
    *(undefined1 *)(param_1 + 0x22) = 3;
    *(undefined1 *)(iVar3 + 0x22) = 2;
    *(int *)(iVar1 + -0x78) = param_1;
  }
  return 1;
}



/* ======================================================================
 * 00002298  lmc_sched_radio_state_set
 * ====================================================================== */

void lmc_sched_radio_state_set(int param_1)

{
  uint *puVar1;
  undefined1 uVar2;
  
  puVar1 = DAT_00002470;
  if (*(char *)(DAT_00002464 + -3) == '\x02') {
    if (param_1 == 0) {
      uVar2 = 3;
    }
    else {
      uVar2 = 4;
    }
    *(undefined1 *)(DAT_00002464 + -3) = uVar2;
    *puVar1 = *puVar1 | 0x80000000;
  }
  return;
}



/* ======================================================================
 * 000022bc  task_22bc
 * ====================================================================== */

void task_22bc(void)

{
  byte *pbVar1;
  uint *puVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  int in_r3;
  int local_18;
  
  local_18 = in_r3;
  evt_flags_set(DAT_00002470,0x200000);
  iVar4 = DAT_00002474;
  iVar3 = DAT_00002468;
  if ((*(byte *)(DAT_00002474 + 0xc) != 0) &&
     (((int)((uint)*(byte *)(DAT_00002474 + 0xc) << 0x1c) < 0 ||
      (txp_scheduler_run(), (int)((uint)*(byte *)(iVar4 + 0xc) << 0x1c) < 0)))) {
    while( true ) {
      txp_dequeue_pending(&local_18);
      iVar5 = local_18;
      if (local_18 == 0) break;
      *(uint *)(local_18 + 0x2c) = *(uint *)(local_18 + 0x2c) | 0x100000;
      txq_list_insert(local_18 + -0x54,*(undefined1 *)(local_18 + -0x47),2);
      *(short *)(DAT_00002478 + 10) = *(short *)(DAT_00002478 + 10) + -1;
      if (*(byte *)(iVar5 + 0x69) < 3) {
        iVar5 = (uint)*(byte *)(iVar5 + 0x69) * 0x3b0 + iVar3;
        *(short *)(iVar5 + 0x30) = *(short *)(iVar5 + 0x30) + -1;
      }
    }
  }
  puVar2 = DAT_00002464;
  if (1 < *(byte *)((int)DAT_00002464 + -3)) {
    pbVar1 = (byte *)((int)DAT_00002464 + -2);
    if (*(byte *)((int)DAT_00002464 + -3) == 4) {
      *(undefined1 *)((int)DAT_00002464 + -3) = 5;
      uVar6 = *DAT_00002464;
      if (0x400 < uVar6) {
        uVar6 = uVar6 - 0x400;
      }
      timer_start((uint)*pbVar1 * 0x3b0 + iVar3 + 0xb0,uVar6);
    }
    iVar3 = DAT_0000247c;
    if ((*(char *)((int)puVar2 + -3) == '\x05') && (*(char *)(iVar4 + 0xc) == '\0')) {
      *(undefined1 *)((int)puVar2 + -3) = 0;
      *(byte *)(iVar3 + 0x15) = *(byte *)(iVar3 + 0x15) & 0xfd;
      puVar2 = DAT_00002464;
      if (*(char *)((int)DAT_00002464 + -0x79) != '\x02') {
        uVar6 = DAT_00002464[-0x1e];
        DAT_00002464[-0x1c] = uVar6;
        lmc_sched_radio_release(uVar6);
        *(undefined1 *)(uVar6 + 0xc) = 0x30;
        return;
      }
      *(undefined1 *)((int)DAT_00002464 + -0x79) = 0;
      uVar6 = puVar2[-0x1e];
      if ((uVar6 != 0) && (*(char *)(uVar6 + 0x22) == '\x04')) {
        *(undefined1 *)(uVar6 + 0x22) = 3;
      }
      syn_scan_abort(0);
    }
  }
  return;
}



/* ======================================================================
 * 000023b0  vif_resume_tx_after_radio
 * ====================================================================== */

undefined4 vif_resume_tx_after_radio(uint param_1)

{
  int iVar1;
  ushort uVar2;
  
  iVar1 = param_1 * 0x3b0 + DAT_00002468;
  lmc_sched_request_radio(iVar1 + 0x44);
  if (((*(char *)(iVar1 + 0x66) == '\x02') || (*(char *)(iVar1 + 0x66) == '\x03')) &&
     (*(short *)(iVar1 + 0x2e) == 0)) {
    *(ushort *)(iVar1 + 0x2e) = *(ushort *)(iVar1 + 0x2c);
    if (param_1 < 2) {
      uVar2 = *(ushort *)(iVar1 + 0x2c) & (~*(ushort *)(iVar1 + 0x15c) | *(ushort *)(iVar1 + 0x160))
      ;
      *(ushort *)(iVar1 + 0x2e) = uVar2;
      *(ushort *)(iVar1 + 0x2e) = uVar2 | *(ushort *)(iVar1 + 0x15e);
    }
    if (*(byte *)(iVar1 + 0x66) < 2) {
      *(undefined1 *)(iVar1 + 0x66) = 3;
    }
  }
  return 1;
}



/* ======================================================================
 * 0000240c  lmc_sched_radio_timer_tick
 * ====================================================================== */

void lmc_sched_radio_timer_tick
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int *piVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  
  piVar1 = DAT_00002464;
  if (*(byte *)((int)DAT_00002464 + -3) < 2) {
    iVar2 = *DAT_00002464;
    iVar4 = DAT_00002464[1];
    iVar3 = fw_read_timer();
    iVar2 = (iVar4 + iVar2) - iVar3;
    if (iVar2 + -0x200 < 1) {
      *(undefined1 *)((int)piVar1 + -3) = 4;
      *piVar1 = 0x100;
      evt_flags_set(DAT_00002470,0x80000000);
      return;
    }
    piVar1[1] = iVar3;
    *piVar1 = iVar2;
    *(undefined1 *)((int)piVar1 + -3) = 2;
    hw_dma_kick((uint)*(byte *)((int)piVar1 + -2),iVar2,
                *(undefined1 *)((uint)*(byte *)((int)piVar1 + -2) * 0x3b0 + DAT_00002468 + 0x24),
                param_1,param_4);
  }
  return;
}



/* ======================================================================
 * 00002480  beacon_tbtt_vif_work
 * ====================================================================== */

void beacon_tbtt_vif_work(int param_1)

{
  bool bVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  undefined2 local_28 [4];
  int local_20;
  int local_1c;
  int *local_18;
  
  iVar4 = param_1 * 0x3b0 + DAT_00002608;
  if ((*(int *)(iVar4 + 0x1c) << 0x1c < 0) && (*(short *)(DAT_0000260c + 0x12) != 0)) {
    *(undefined2 *)(DAT_00002610 + 8) = 1;
    evt_flags_set(DAT_00002614,0x8000);
  }
  local_18 = (int *)(iVar4 + 0x118);
  local_20 = *local_18 + -4000;
  local_1c = iVar4 + 0xd8;
  timer_start(local_1c,local_20);
  iVar5 = DAT_00002608;
  uVar2 = *(uint *)(iVar4 + 0x1c);
  *(uint *)(iVar4 + 0x1c) = uVar2 & 0xffefffff;
  iVar5 = param_1 * 0x3b0 + iVar5;
  if ((((int)(uVar2 * 0x20000000) < 0) && (*(short *)(iVar5 + 0x15c) != 0)) &&
     (*(char *)(iVar5 + 0x164) == '\0')) {
    bVar1 = false;
    iVar3 = param_1 * 0x70 + DAT_00002618;
    iVar3 = ie_find_in_frame(*(undefined4 *)(iVar3 + 0x10),*(undefined2 *)(iVar3 + 0x18),5,0);
    if (((iVar3 != 0) && (3 < *(byte *)(iVar3 + 1))) &&
       (((*(byte *)(iVar3 + 4) & 1) != 0 &&
        (evt_flags_set(DAT_00002614,0x100), *(char *)(DAT_0000261c + 0x15) != '\0')))) {
      local_28[0] = 9;
      ind_080c_suspend_resume(*(undefined1 *)(iVar4 + 0x1a),local_28);
      bVar1 = true;
    }
    if (((int)(*(uint *)(iVar4 + 0x1c) << 0xc) < 0) && (!bVar1)) {
      *(uint *)(iVar4 + 0x1c) = *(uint *)(iVar4 + 0x1c) & 0xfff7ffff;
      local_28[0] = 1;
      ind_080c_suspend_resume(*(undefined1 *)(iVar4 + 0x1a),local_28);
    }
  }
  if (*(char *)(iVar4 + 0x18) == '\x06') {
    *(uint *)(iVar4 + 0x1c) = *(uint *)(iVar4 + 0x1c) & 0x9fffffff;
    if ((int)((uint)*(byte *)(iVar4 + 0x1ac) << 0x18) < 0) {
      ps_arm_listen_interval_timer(param_1);
    }
    else {
      *(undefined1 *)(param_1 * 0x98 + DAT_00002618 + 0x493) = 0;
    }
    if ((int)(*(uint *)(iVar4 + 0x1c) << 4) < 0) {
      if (*(char *)(iVar5 + 0x165) == '\0') {
        *(uint *)(iVar4 + 0x1c) = *(uint *)(iVar4 + 0x1c) & 0xf7ffffff;
        timer_start(local_1c,(uint)*(byte *)(iVar4 + 0x181) * *local_18 + local_20);
      }
      else {
        *(char *)(iVar5 + 0x165) = *(char *)(iVar5 + 0x165) + -1;
      }
    }
  }
  if ((*(short *)(iVar4 + 0x30) != 0) || (*(int *)(iVar4 + 0x1c) << 5 < 0)) {
    *(uint *)(iVar4 + 0x1c) = *(uint *)(iVar4 + 0x1c) & 0xfbffffff;
    evt_flags_set(DAT_00002614,0x200000);
  }
  if ((*(char *)(DAT_00002610 + 0x47) == '\x01') && (*(byte *)(DAT_00002610 + 0xbd) < 2)) {
    *(undefined1 *)(DAT_00002610 + 0x47) = 2;
    txp_prepare_all_pipes_idle();
    tx_arm_timeout((char *)(iVar4 + 0x18),DAT_00002620);
  }
  return;
}



/* ======================================================================
 * 00002624  pipe_clear_entry
 * ====================================================================== */

void pipe_clear_entry(int param_1)

{
  int iVar1;
  
  iVar1 = DAT_0000282c;
  *(undefined4 *)(param_1 * 0x44 + DAT_00002828 + 0x718) = 0;
  *(undefined4 *)(param_1 * 0x20 + iVar1) = 0;
  iVar1 = param_1 * 0xc + DAT_00002830;
  *(undefined4 *)(iVar1 + 0x20) = 0xffffffff;
  *(undefined4 *)(iVar1 + 0x24) = 0xff;
  *(undefined4 *)(iVar1 + 0x28) = 0xff;
  return;
}



/* ======================================================================
 * 00002652  pipe_setup_entry
 * ====================================================================== */

void pipe_setup_entry(int param_1,int param_2,undefined4 param_3,undefined4 *param_4,int param_5)

{
  uint uVar1;
  uint uVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  undefined4 unaff_r9;
  bool bVar9;
  bool bVar10;
  undefined4 in_cr0;
  
  iVar4 = param_1 * 0x44 + DAT_00002828;
  *(undefined2 *)(iVar4 + 0x6dc) = *(undefined2 *)param_4;
  *(undefined2 *)(iVar4 + 0x6de) = *(undefined2 *)((int)param_4 + 2);
  *(undefined2 *)(iVar4 + 0x6e0) = *(undefined2 *)(param_4 + 1);
  *(undefined1 *)(iVar4 + 0x700) = (char)param_3;
  iVar6 = param_1 * 0xc + DAT_00002830;
  *(undefined4 *)(iVar6 + 0x20) = *param_4;
  *(uint *)(iVar6 + 0x24) = (uint)*(ushort *)(param_4 + 1);
  *(undefined4 *)(iVar6 + 0x28) = param_3;
  uVar7 = param_1 * 2 + 0x10;
  uVar8 = 3 << (uVar7 & 0xff);
  uVar2 = *(uint *)(DAT_0000283c + 0x14) & ~uVar8 |
          1 << (param_1 * 2 +
                (uint)(*(char *)(param_2 * 0x98 + DAT_00002834 + 0x481) !=
                      *(char *)(DAT_00002838 + 0x19)) + 0x10 & 0xff);
  *(uint *)(DAT_0000283c + 0x14) = uVar2;
  *(uint *)(DAT_00002830 + 0xdc) = uVar2;
  uVar1 = param_1 * 0x20 + DAT_0000282c;
  *(undefined4 *)(uVar1 + 0x10) = 0;
  *(undefined4 *)(uVar1 + 0x14) = 0;
  *(undefined4 *)(uVar1 + 0x18) = 0;
  *(undefined4 *)(uVar1 + 0x1c) = 0;
  *(undefined4 *)(iVar4 + 0x704) = 0;
  uVar2 = DAT_00002840;
  uVar3 = param_5 * 0x10;
  bVar9 = CARRY4(uVar3,DAT_00002840);
  bVar10 = SCARRY4(uVar3,DAT_00002840);
  uVar3 = uVar3 + DAT_00002840;
  *(uint *)(uVar1 + 4) = uVar3;
  if (uVar3 != 0) {
    coprocessor_storelong(4,in_cr0,unaff_r9);
  }
  if (bVar10) {
    uVar7 = uVar3 & uVar3 >> 3;
  }
  if ((int)uVar3 < 0 != bVar10) {
    uVar1 = uVar1 + 0xdf0;
  }
  if (bVar9) {
    *(undefined1 **)(uVar8 - ((int)uVar2 >> 0x1c)) = (undefined1 *)(iVar4 + 0x700);
  }
  do {
    iVar6 = uVar1 * 0x44 + DAT_00002828;
    iVar4 = iVar6 + uVar8;
    if (*(int *)(iVar4 + 0x18) != 0) {
      uVar2 = 0;
      do {
        iVar5 = uVar2 * 4;
        uVar2 = uVar2 + 1;
        *(undefined4 *)(iVar6 + iVar5 + uVar8 + 8) =
             *(undefined4 *)(uVar1 * 0x20 + iVar5 + uVar7 + 0x10);
      } while (uVar2 < 4);
      *(undefined4 *)(iVar4 + 0x1c) = *(undefined4 *)(uVar1 * 0x20 + uVar7 + 4);
    }
    uVar1 = uVar1 + 1;
  } while (uVar1 < 4);
  return;
}



/* ======================================================================
 * 0000275e  stats_export_pipe_counters
 * ====================================================================== */

void stats_export_pipe_counters(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  undefined4 *puVar6;
  int iVar7;
  
  iVar2 = DAT_00002830;
  iVar1 = DAT_00002828;
  uVar3 = 0;
  do {
    iVar4 = uVar3 * 0x44 + iVar1;
    if (*(int *)(iVar4 + 0x718) == 0) {
      iVar4 = uVar3 * 0xc + iVar2;
      *(undefined4 *)(iVar4 + 0x20) = 0xffffffff;
      *(undefined4 *)(iVar4 + 0x24) = 0xff;
      *(undefined4 *)(iVar4 + 0x28) = 0xff;
    }
    else {
      uVar5 = ((undefined4 *)(iVar4 + DAT_00002844))[1];
      iVar7 = uVar3 * 0xc + iVar2;
      *(undefined4 *)(iVar7 + 0x20) = *(undefined4 *)(iVar4 + DAT_00002844);
      *(uint *)(iVar7 + 0x24) = uVar5 & 0xffff;
      *(uint *)(iVar7 + 0x28) = (uint)*(byte *)(iVar4 + 0x700);
      puVar6 = (undefined4 *)(uVar3 * 0x20 + DAT_0000282c);
      puVar6[4] = *(undefined4 *)(iVar4 + 0x708);
      puVar6[5] = *(undefined4 *)(iVar4 + 0x70c);
      puVar6[6] = *(undefined4 *)(iVar4 + 0x710);
      puVar6[7] = *(undefined4 *)(iVar4 + 0x714);
      puVar6[1] = *(undefined4 *)(iVar4 + 0x71c);
      *puVar6 = *(undefined4 *)(iVar4 + 0x718);
    }
    uVar3 = uVar3 + 1;
  } while (uVar3 < 4);
  *(undefined4 *)(DAT_00002830 + 0xdc) = *(undefined4 *)(DAT_0000283c + 0x14);
  return;
}



/* ======================================================================
 * 000027d4  txq_remove_frame_by_link_seq
 * ====================================================================== */

void txq_remove_frame_by_link_seq(int param_1,uint param_2,uint param_3)

{
  int iVar1;
  int *piVar2;
  undefined2 uVar3;
  int *piVar4;
  int *piVar5;
  
  iVar1 = DAT_00002848;
  piVar2 = *(int **)(DAT_00002848 + 0x10);
  piVar5 = (int *)0x0;
  while( true ) {
    piVar4 = piVar2;
    if (piVar4 == (int *)0x0) {
      return;
    }
    if ((*(byte *)(piVar4 + 0x1b) == param_2) && (*(ushort *)(*piVar4 + 0x16) >> 4 == param_3))
    break;
    piVar2 = (int *)piVar4[0x10];
    piVar5 = piVar4;
  }
  if (piVar5 == (int *)0x0) {
    *(int *)(DAT_00002848 + 0x10) = piVar4[0x10];
  }
  else {
    piVar5[0x10] = piVar4[0x10];
  }
  if (*(int **)(iVar1 + 0x14) == piVar4) {
    *(int **)(iVar1 + 0x14) = piVar5;
  }
  if (param_1 == 0) {
    uVar3 = 0;
  }
  else {
    if (param_1 != 1) {
      return;
    }
    uVar3 = 0xb;
  }
  *(undefined2 *)(piVar4 + 7) = uVar3;
  tx_ctx_free_locked(piVar4);
  return;
}



/* ======================================================================
 * 00002864  phy_resume_state4
 * ====================================================================== */

void phy_resume_state4(void)

{
  phy_rx_enable();
  *(undefined1 *)(DAT_00002bcc + 0x16) = 4;
  txp_scheduler_run();
  return;
}



/* ======================================================================
 * 00002876  regs_save_ctx
 * ====================================================================== */

void regs_save_ctx(void)

{
  undefined4 *puVar1;
  undefined4 *puVar2;
  
  puVar2 = DAT_00002bd4;
  puVar1 = DAT_00002bd0;
  DAT_00002bd4[1] = *DAT_00002bd0;
  puVar2[2] = puVar1[1];
  puVar2[3] = (uint)*(ushort *)(DAT_00002bd8 + 0xe);
  puVar2[4] = puVar1[2];
  *puVar2 = puVar1[3];
  return;
}



/* ======================================================================
 * 00002892  mac_reprogram_after_channel
 * ====================================================================== */

void mac_reprogram_after_channel(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  iVar1 = DAT_00002bdc;
  *(undefined1 *)(DAT_00002bdc + 0x1d) = 0;
  *DAT_00002be4 = DAT_00002be0 & 0xf6ffffff;
  iVar2 = DAT_00002be8;
  uVar3 = 0;
  do {
    if (*(char *)(uVar3 * 0x98 + iVar2 + 0x470) == '\x02') {
      pas_program_rate_tables();
    }
    uVar3 = uVar3 + 1 & 0xff;
  } while (uVar3 < 3);
  mac_program_ifs_timing();
  txp_build_ctl_frame(0,DAT_00002bec,0xd4);
  txp_build_ctl_frame(1,DAT_00002bec + 0x54,0xd4);
  txp_build_ctl_frame(0,DAT_00002bec + 0xa8,0xc4);
  txp_build_ctl_frame(1,DAT_00002bec + 0xfc,0xc4);
  *DAT_00002bf0 = *(undefined4 *)(iVar1 + 0x24);
  regs_save_ctx();
  return;
}



/* ======================================================================
 * 00002908  mac_reinit_after_wake
 * ====================================================================== */

void mac_reinit_after_wake(void)

{
  char cVar1;
  int iVar2;
  undefined4 *puVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  undefined4 in_r3;
  int iVar7;
  int iVar8;
  
  iVar2 = DAT_00002bdc;
  if (*(char *)(DAT_00002bdc + 0x1e) != '\0') {
    *(undefined1 *)(DAT_00002bdc + 0x1e) = 0;
    rx_subsystem_init();
    mac_hw_init_pipes();
    sync_regs_10();
    iVar4 = DAT_00002bf8;
    iVar7 = DAT_00002bf4;
    uVar5 = 0;
    iVar8 = DAT_00002be8 + -0x98;
    do {
      iVar6 = uVar5 * 2;
      uVar5 = uVar5 + 1;
      *(undefined2 *)(iVar6 + iVar4 + iVar7) = *(undefined2 *)(iVar6 + iVar8 + 0x90);
    } while (uVar5 < 2);
    uVar5 = 0;
    do {
      iVar7 = uVar5 * 0x3b0 + DAT_00002bfc;
      cVar1 = *(char *)(iVar7 + 0x18);
      if (((cVar1 == '\x06') || (cVar1 == '\x05')) && (*(char *)(iVar7 + 0x3c6) != '\0')) {
        mac_program_bssid(uVar5 * 6 + DAT_00002be8 + 0x460,1,0x3a0,6,in_r3);
        puVar3 = DAT_00002bf0;
        DAT_00002bf0[0x16] = 0x2000000;
        puVar3[0x12] = &DAT_02000001;
        break;
      }
      uVar5 = uVar5 + 1;
    } while (uVar5 < 2);
    txp_build_tbtt_desc(DAT_00002c00,1,1);
    txp_build_tbtt_desc(DAT_00002c04,1,1);
    txp_build_tbtt_desc(DAT_00002c04 + 0x54,1,1);
    txp_build_tbtt_desc(DAT_00002bec + -0xa8,1,1);
    txp_build_tbtt_desc(DAT_00002bec + -0x54,1,1);
    txp_build_tbtt_desc(DAT_00002bec,1,1);
    txp_build_tbtt_desc(DAT_00002bec + 0x54,1,1);
    txp_build_tbtt_desc(DAT_00002bec + 0xa8,1,1);
    txp_build_tbtt_desc(DAT_00002c00 + -0x54,1,1);
    txp_build_tbtt_desc(DAT_00002c00 + 0xfc,1,1);
    txp_build_tbtt_desc(DAT_00002c08,1,1);
    uVar5 = 0;
    do {
      iVar7 = uVar5 * 4;
      uVar5 = uVar5 + 1;
      *(undefined4 *)(iVar7 + iVar4 + 0x7000) = *(undefined4 *)(iVar7 + iVar8 + 0x10);
    } while (uVar5 < 0x20);
    *DAT_00002c10 = DAT_00002c0c & 0xf6ffffff;
    do {
    } while (-1 < *(int *)(DAT_00002c14 + 0x20));
    *DAT_00002c18 = 0x1000000;
    iVar7 = DAT_00002c1c;
    *(undefined4 *)(DAT_00002c1c + 0x10) = 5;
    *(undefined4 *)(iVar7 + 0x14) = 0x23;
    *(undefined4 *)(iVar7 + 0x18) = 0x30;
    lmc_msg_pool_init();
    if (*(short *)(DAT_00002bcc + 0x10) != 0) {
      *(undefined1 *)(iVar2 + 0x1d) = 1;
      mac_program_slot_timings
                (*(undefined2 *)(DAT_00002bc8 + 2),*(undefined4 *)(DAT_00002bc8 + 0x1c));
      txp_install_resp_descs(3,0);
      stats_export_pipe_counters();
      if (*(int *)(iVar2 + 0x28) != 0) {
        mac_program_bssid(*(int *)(iVar2 + 0x28),3);
      }
      mac_program_mode_regs();
      puVar3 = DAT_00002bf0;
      *(undefined4 *)(iVar2 + 0x24) = *DAT_00002bf0;
      *puVar3 = DAT_00002c20;
    }
  }
  return;
}



/* ======================================================================
 * 00002a92  phy_maybe_kick
 * ====================================================================== */

void phy_maybe_kick(void)

{
  if (*(char *)(DAT_00002bcc + 0x16) == '\x02') {
    phy_wake_sequence();
  }
  return;
}



/* ======================================================================
 * 00002ad8  regs_restore_ctx
 * ====================================================================== */

void regs_restore_ctx(void)

{
  undefined4 *puVar1;
  undefined4 *puVar2;
  
  puVar2 = DAT_00002bd4;
  puVar1 = DAT_00002bd0;
  *DAT_00002bd0 = DAT_00002bd4[1];
  puVar1[1] = puVar2[2];
  puVar1[3] = *puVar2;
  puVar1[2] = puVar2[5];
  return;
}



/* ======================================================================
 * 00002aee  mac_set_channel
 * ====================================================================== */

void mac_set_channel(uint param_1)

{
  int iVar1;
  undefined4 uVar2;
  bool bVar3;
  
  iVar1 = DAT_00002bcc;
  if (*(ushort *)(DAT_00002bcc + 0x10) != param_1) {
    phy_rx_disable_and_drain();
    rx_handler_main_loop();
    bVar3 = -1 < (int)((uint)*DAT_00002bfc << 0x1a);
    if (bVar3) {
      uVar2 = 2;
    }
    else {
      uVar2 = 3;
    }
    phy_cal_set_channel_and_arm(uVar2,param_1,bVar3);
    *(short *)(iVar1 + 0x10) = (short)param_1;
    phy_rx_enable();
  }
  return;
}



/* ======================================================================
 * 00002b24  phy_enter_state3
 * ====================================================================== */

void phy_enter_state3(void)

{
  phy_rx_disable_and_drain();
  *(undefined1 *)(DAT_00002bcc + 0x16) = 3;
  return;
}



/* ======================================================================
 * 00002b32  mac_set_ps_bit
 * ====================================================================== */

void mac_set_ps_bit(int param_1,int param_2)

{
  int iVar1;
  undefined4 *puVar2;
  byte bVar3;
  uint uVar4;
  byte *pbVar5;
  
  iVar1 = DAT_00002bc8;
  uVar4 = (uint)(*(char *)(param_1 * 0x98 + DAT_00002be8 + 0x481) != *(char *)(DAT_00002bcc + 0x79))
  ;
  pbVar5 = (byte *)(DAT_00002bc8 + param_1 + 0x440);
  if (param_2 == 0) {
    *pbVar5 = *pbVar5 & 0xef;
    bVar3 = *(byte *)(iVar1 + uVar4 + 0x444) & 0xef;
  }
  else {
    *pbVar5 = *pbVar5 | 0x10;
    bVar3 = *(byte *)(iVar1 + uVar4 + 0x444) | 0x10;
  }
  *(byte *)(iVar1 + uVar4 + 0x444) = bVar3;
  iVar1 = DAT_00002c08;
  puVar2 = DAT_00002bdc;
  *(undefined4 *)(DAT_00002c08 + 0x54) = *DAT_00002bdc;
  *(undefined4 *)(iVar1 + 0x58) = puVar2[1];
  return;
}



/* ======================================================================
 * 00002b96  phy_enter_state3_checked
 * ====================================================================== */

void phy_enter_state3_checked(void)

{
  int iVar1;
  
  phy_enter_state3();
  iVar1 = DAT_00002bc8;
  if (*(int *)(DAT_00002c2c + 0x30) != 0) {
    *(undefined1 *)(DAT_00002bc8 + 0xb) = 1;
    *(undefined1 *)(iVar1 + 10) = 1;
    if ((*(uint *)(DAT_00002bdc + -4) & DAT_00002c30) != DAT_00002c34) {
      *(undefined1 *)(iVar1 + 10) = 0;
      task_df4();
    }
  }
  return;
}



/* ======================================================================
 * 00002c38  tsf_sync_from_beacon
 * ====================================================================== */

void tsf_sync_from_beacon(int *param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  int iVar8;
  char cVar9;
  longlong lVar10;
  longlong lVar11;
  undefined8 uVar12;
  
  iVar6 = param_1[2];
  iVar1 = get_link_word_78(*(undefined1 *)((int)param_1 + 7),*(undefined1 *)((int)param_1 + 6));
  lVar10 = tsf_add_offset();
  iVar3 = (int)((ulonglong)lVar10 >> 0x20);
  param_1[2] = (int)lVar10;
  iVar8 = DAT_00002d30;
  param_1[3] = iVar3;
  uVar7 = 0;
  while (((iVar2 = uVar7 * 0x98 + iVar8, *(byte *)(iVar2 + 0x470) < 2 ||
          ((((cVar9 = *(char *)(iVar2 + 0x472), cVar9 != '\x01' && (cVar9 != '\x05')) &&
            (cVar9 != '\x02')) ||
           ((iVar5 = *param_1, *(short *)(iVar5 + 0x10) != *(short *)(iVar2 + 0x482) ||
            (*(short *)(iVar5 + 0x12) != *(short *)(iVar2 + 0x484))))))) ||
         (*(short *)(iVar5 + 0x14) != *(short *)(iVar2 + 0x486)))) {
    uVar7 = uVar7 + 1 & 0xff;
    if (1 < uVar7) {
      return;
    }
  }
  if (1 < uVar7) {
    return;
  }
  lVar11 = u64_add_u32(0,*(undefined4 *)(*param_1 + 0x1c),*(undefined4 *)(iVar5 + 0x18));
  if ((int)((uint)*(byte *)(iVar2 + 0x471) << 0x1c) < 0) {
    *(longlong *)(iVar2 + 0x488) = lVar11 - lVar10;
    tsf_nudge_away_from_tbtt(uVar7);
    return;
  }
  lVar11 = lVar11 + *(longlong *)(iVar2 + 0x488);
  uVar4 = (undefined4)((ulonglong)lVar11 >> 0x20);
  cVar9 = 1 < *(byte *)(iVar2 + 0x472);
  if ((*(byte *)(iVar2 + 0x472) == 2) &&
     (u64_cmp((int)lVar10,iVar3,(int)lVar11,uVar4), cVar9 != '\0')) {
    return;
  }
  tsf_adjust_small((int)lVar11,uVar4,iVar6 + iVar1);
  if (*(byte *)(DAT_00002d34 + 0x1a) == uVar7) {
    return;
  }
  iVar8 = (uint)*(byte *)(DAT_00002d34 + 0x1a) * 0x98 + iVar8;
  uVar12 = u64_rsub((int)(lVar11 - *(longlong *)(param_1 + 2)),
                    (int)((ulonglong)(lVar11 - *(longlong *)(param_1 + 2)) >> 0x20),
                    *(undefined4 *)(iVar8 + 0x488),*(undefined4 *)(iVar8 + 0x48c));
  *(undefined8 *)(iVar8 + 0x488) = uVar12;
  return;
}



/* ======================================================================
 * 00002d38  tsf_align_to_beacon_interval
 * ====================================================================== */

void tsf_align_to_beacon_interval(int param_1)

{
  int iVar1;
  uint *puVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  undefined8 uVar6;
  
  uVar6 = tsf_read();
  iVar5 = *(int *)(param_1 * 0x3b0 + DAT_00002f0c + 0x118);
  if (iVar5 == 0) {
    iVar4 = 0;
  }
  else {
    iVar4 = iVar5;
    __udivmoddi4((int)uVar6,(int)((ulonglong)uVar6 >> 0x20),iVar5,0);
  }
  iVar1 = DAT_00002f10;
  *(int *)(DAT_00002f10 + 0x2c) = (int)uVar6 + (iVar5 - iVar4);
  beacon_build_tx_desc(param_1);
  *(undefined4 *)(iVar1 + 0x28) = 3;
  irq_fiq_disable_save();
  puVar2 = DAT_00002f14;
  if ((*(uint *)(iVar1 + 0x30) & 1) == 0) {
    uVar3 = *(uint *)(iVar1 + 0x30) | 1;
    *(uint *)(iVar1 + 0x30) = uVar3;
    *puVar2 = uVar3;
  }
  *(uint *)(DAT_00002f18 + 8) = (*(uint *)(iVar1 + 0x2c) & 0xffffff) + 0x40000000;
  irq_fiq_restore();
  return;
}



/* ======================================================================
 * 00002da0  task_2da0
 * ====================================================================== */

void task_2da0(void)

{
  int iVar1;
  
  iVar1 = DAT_00002f1c;
  if (*(int *)(DAT_00002f10 + 0x28) == 4) {
    tsf_align_to_beacon_interval();
  }
  else if (*(char *)(DAT_00002f1c + 0x19) != '\0') {
    beacon_tbtt_vif_work(*(undefined1 *)(DAT_00002f1c + 0x18));
    if (*(char *)(DAT_00002f10 + 0x5f) != '\0') {
      *(char *)(DAT_00002f10 + 0x5f) = *(char *)(DAT_00002f10 + 0x5f) + -1;
    }
    *(undefined1 *)(iVar1 + 0x19) = 0;
    return;
  }
  return;
}



/* ======================================================================
 * 00002dd2  tsf_nudge_away_from_tbtt
 * ====================================================================== */

void tsf_nudge_away_from_tbtt(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  undefined4 uVar2;
  undefined4 uVar3;
  uint uVar4;
  int iVar5;
  int iVar6;
  undefined8 uVar7;
  longlong lVar8;
  undefined8 uVar9;
  
  iVar6 = DAT_00002f24;
  if (*(char *)((uint)*(byte *)(DAT_00002f20 + 0x19) * 0x98 + DAT_00002f24 + 0x472) != '\x02') {
    uVar7 = tsf_read(2);
    iVar5 = 0;
    uVar4 = *(uint *)(param_1 * 0x3b0 + DAT_00002f0c + 0x118);
    uVar1 = uVar4;
    __udivmoddi4((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),uVar4,0,uVar7,param_4);
    lVar8 = u64_add_u32((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),uVar4 - uVar1);
    iVar6 = param_1 * 0x98 + iVar6;
    uVar3 = *(undefined4 *)(iVar6 + 0x48c);
    uVar2 = *(undefined4 *)(iVar6 + 0x488);
    lVar8 = lVar8 + *(longlong *)(iVar6 + 0x488);
    uVar1 = uVar4;
    __udivmoddi4((int)lVar8,(int)((ulonglong)lVar8 >> 0x20),uVar4,0);
    if (uVar1 < 0x6400) {
      iVar5 = 0x6400 - uVar1;
    }
    if (uVar4 < uVar1 + 0x6400) {
      iVar5 = (uVar4 - uVar1) + 0x6400;
    }
    if (iVar5 != 0) {
      uVar9 = u64_add_u32(uVar2,uVar3,iVar5);
      *(undefined8 *)(iVar6 + 0x488) = uVar9;
      u64_sub_u32((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),iVar5);
      tsf_write();
    }
  }
  return;
}



/* ======================================================================
 * 00002e7c  tsf_align_request
 * ====================================================================== */

undefined4 tsf_align_request(undefined1 param_1)

{
  int iVar1;
  int iVar2;
  
  iVar2 = DAT_00002f1c;
  iVar1 = DAT_00002f10;
  if (*(int *)(DAT_00002f10 + 0x28) != 0) {
    if ((*(char *)(DAT_00002f1c + 0x19) != '\0') || (*(int *)(DAT_00002f10 + 0x28) != 1)) {
      return 0;
    }
    *(undefined1 *)(DAT_00002f1c + 0x19) = 1;
    *(undefined1 *)(iVar2 + 0x18) = param_1;
    *(undefined4 *)(iVar1 + 0x28) = 2;
    tsf_align_to_beacon_interval();
  }
  return 1;
}



/* ======================================================================
 * 00002eae  beacon_build_tx_desc
 * ====================================================================== */

void beacon_build_tx_desc(int param_1)

{
  uint *puVar1;
  ushort uVar2;
  short sVar3;
  
  puVar1 = DAT_00002f28;
  *DAT_00002f28 =
       *(uint *)(DAT_00002f2c + 0x38) |
       (*(uint *)(DAT_00002f2c + 0x38) & 0x1fff) << 0x10 | DAT_00002f30;
  if ((*(char *)(DAT_00002f20 + 0x92) == '\x02') || (*(char *)(DAT_00002f20 + 0x92) == '\x03')) {
    uVar2 = fw_rand_masked((uint)*(ushort *)(DAT_00002f20 + 0xee) * 2 + 1);
    sVar3 = (uVar2 & 0x3ff) + 0xbc00;
  }
  else {
    sVar3 = (short)DAT_00002f34;
  }
  *(short *)((int)puVar1 + 6) = sVar3;
  puVar1[2] = 0x58000000;
  txp_build_beacon_pipe_words(puVar1 + 3,param_1 * 0x70 + DAT_00002f24 + 0x10,0);
  return;
}



/* ======================================================================
 * 00002f38  txp_build_beacon_pipe_words
 * ====================================================================== */

void txp_build_beacon_pipe_words(int *param_1,uint *param_2,uint param_3)

{
  uint uVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  uint local_28;
  uint local_24;
  int *piStack_20;
  uint *puStack_1c;
  uint local_18;
  
  uVar1 = param_2[2];
  piStack_20 = param_1;
  puStack_1c = param_2;
  local_18 = param_3;
  iVar2 = pas_rate_to_hw_code(*(undefined1 *)((int)param_2 + 0xf));
  pas_build_phy_rate_words
            (&local_24,&local_28,*(undefined1 *)((int)param_2 + 0xf),param_2[1],
             *(undefined1 *)((int)param_2 + 0xd));
  if ((local_28 & 0x1fff) >> 10 == 5) {
    uVar3 = pac_phy_calc_duration(*(undefined1 *)((int)param_2 + 0xf),(ushort)param_2[2] + 4);
    local_24 = (uVar3 & 0xfff) << 0xc | local_24;
  }
  *param_1 = (local_28 & 0xffffff) + 0x51000000;
  param_1[1] = (local_24 & 0xffffff) + 0x50000000;
  param_1[2] = ((ushort)uVar1 + 4 & 0xffff) + 0x52000000 | iVar2 << 0x10;
  uVar3 = param_2[1];
  uVar4 = (uint)*(ushort *)((int)param_2 + 10);
  if ((int)(uVar3 << 0x1b) < 0) {
    uVar4 = uVar4 | 0x800;
  }
  param_1[3] = uVar4 + 0x32000000;
  param_1[4] = (local_18 & 0xffffff) + 0x32000000;
  param_1[5] = (*param_2 + 4 & 0x7fffff) + 0x29000000;
  if ((uVar3 & 1) == 0) {
    iVar2 = ((uint)*(byte *)((int)param_2 + 0x6a) * 2 + DAT_00003038 + DAT_0000303c & 0x7fffff) +
            0x21000000;
  }
  else {
    iVar2 = *(ushort *)(*param_2 + 0x16) + 0x32000000;
  }
  param_1[6] = iVar2;
  param_1[7] = DAT_00003040;
  uVar3 = *param_2;
  param_1[8] = (uVar3 + 0x20 & 0x7fffff) + 0x40000000;
  param_1[9] = ((ushort)uVar1 - 0x20 & 0xfff) << 0xc | uVar3 & 3;
  param_1[10] = DAT_00003044;
  param_1[0xb] = -0x10000000;
  return;
}



/* ======================================================================
 * 00003048  wsm_h_1C_map_link_impl
 * ====================================================================== */

void wsm_h_1C_map_link_impl(undefined2 *param_1)

{
  undefined4 uVar1;
  
  ap_map_link(param_1 + 2);
  *param_1 = 8;
  uVar1 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00003064  ie_index_build
 * ====================================================================== */

undefined4 ie_index_build(int param_1,int param_2,uint param_3)

{
  bool bVar1;
  int iVar2;
  uint *puVar3;
  uint *puVar4;
  undefined4 uVar5;
  uint *puVar6;
  uint uVar7;
  
  bVar1 = true;
  uVar5 = 0;
  if (((param_2 != 0) && (0x24 < param_3)) && ((int)param_3 < 700)) {
    iVar2 = DAT_0000346c * param_1 + DAT_00003470;
    puVar6 = (uint *)(iVar2 + 0x4c8);
    puVar3 = (uint *)(iVar2 + 0x2c4);
    puVar4 = puVar3;
    if (*(int *)(param_1 * DAT_0000346c + DAT_00003470 + 0x2c0) == 0) {
      puVar4 = puVar6;
      puVar6 = puVar3;
    }
    *puVar4 = 0;
    for (uVar7 = param_2 + 0x24; uVar7 < (param_2 + param_3) - 1;
        uVar7 = (uint)*(byte *)(uVar7 + 1) + uVar7 + 2) {
      if (0xff < *puVar4) goto LAB_000030fe;
      *(short *)((int)puVar4 + *puVar4 * 2 + 4) = (short)(uVar7 - param_2);
      bVar1 = (bool)(bVar1 & (uVar7 - param_2 & 0xffff) ==
                             (uint)*(ushort *)((int)puVar6 + *puVar4 * 2 + 4));
      *puVar4 = *puVar4 + 1;
    }
    uVar5 = 0;
    if ((bVar1) && (*puVar6 == *puVar4)) {
LAB_000030fe:
      uVar5 = 1;
    }
  }
  return uVar5;
}



/* ======================================================================
 * 00003104  ie_find_vendor_match
 * ====================================================================== */

int ie_find_vendor_match(int param_1,uint param_2,int param_3)

{
  byte bVar1;
  int iVar2;
  byte bVar3;
  uint uVar4;
  
  if (((param_1 != 0) && (param_2 != 0)) && (param_3 != 0)) {
    bVar1 = *(byte *)(param_1 + 1);
    for (uVar4 = 0; uVar4 < param_2; uVar4 = uVar4 + 1) {
      iVar2 = uVar4 * 8 + param_3;
      bVar3 = (*(byte *)(iVar2 + 1) >> 4) + 3;
      if ((bVar3 <= bVar1) && (iVar2 = fw_mem_equal(iVar2 + 2,bVar3,param_1 + 2,bVar3), iVar2 == 1))
      {
        return uVar4 * 8 + param_3;
      }
    }
  }
  return 0;
}



/* ======================================================================
 * 00003156  beacon_filter_ie_diff
 * ====================================================================== */

/* WARNING: Type propagation algorithm not settling */

undefined4 beacon_filter_ie_diff(int param_1,int param_2,uint param_3)

{
  byte bVar1;
  int iVar2;
  int *piVar3;
  uint uVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  int iVar9;
  int iVar10;
  uint *puVar11;
  uint *puVar12;
  uint uVar13;
  bool bVar14;
  bool bVar15;
  uint local_58;
  uint local_54;
  uint *local_48;
  uint local_3c;
  int local_28;
  int local_24;
  
  bVar14 = param_2 == 0;
  local_3c = 0;
  do {
    if ((bVar14) || (param_3 < 0x25)) {
      return 0;
    }
    bVar15 = SBORROW4(param_3,700);
    bVar14 = (int)(param_3 - 700) < 0;
LAB_00003172:
    if (bVar14 == bVar15) {
      return 0;
    }
    iVar2 = param_1 * 0x188 + DAT_00003474;
    piVar3 = (int *)(iVar2 + 0x48);
    bVar14 = true;
  } while (*piVar3 == 0);
  iVar10 = DAT_0000346c * param_1 + DAT_00003470;
  iVar9 = DAT_0000346c * param_1 + DAT_00003470;
  puVar11 = (uint *)(iVar9 + 0x2c4);
  local_48 = (uint *)(iVar9 + 0x4c8);
  puVar12 = puVar11;
  if (*(int *)(iVar10 + 0x2c0) == 0) {
    puVar12 = local_48;
    local_48 = puVar11;
  }
  for (; local_3c < *local_48; local_3c = local_3c + 1) {
    uVar4 = (uint)*(ushort *)((int)local_48 + local_3c * 2 + 4);
    iVar9 = iVar10 + uVar4;
    uVar8 = (uint)*(byte *)(iVar9 + 4);
    if (uVar8 == 0xdd) {
      local_24 = iVar2 + 0x148;
      local_28 = iVar2 + 0x150;
      local_54 = ie_find_vendor_match(iVar9 + 4,*(undefined4 *)(iVar2 + 0x14c));
      if (local_54 != 0) {
        for (uVar13 = 0; uVar13 < *puVar12; uVar13 = uVar13 + 1) {
          local_58 = (uint)*(ushort *)((int)puVar12 + uVar13 * 2 + 4);
          iVar6 = local_58 - param_3;
          uVar5 = local_58;
LAB_00003208:
          bVar15 = SBORROW4(uVar5,param_3);
          bVar14 = iVar6 < 0;
          if (bVar14 == bVar15) goto LAB_00003172;
          if (((*(byte *)(param_2 + local_58) == uVar8) &&
              (uVar5 = ie_find_vendor_match(param_2 + uVar4,*(undefined4 *)(local_24 + 4),local_28),
              uVar5 == local_54)) && ((*(byte *)(local_54 + 1) & 1) != 0)) {
            iVar9 = fw_mem_equal(iVar9 + 6,*(undefined1 *)(iVar9 + 5),param_2 + local_58 + 2,
                                 *(undefined1 *)(param_2 + local_58 + 1));
            if (iVar9 == 0) {
              return 1;
            }
            goto LAB_00003346;
          }
        }
        bVar1 = *(byte *)(local_54 + 1);
LAB_00003342:
        if ((int)((uint)bVar1 << 0x1e) < 0) {
          return 1;
        }
      }
    }
    else if ((*(byte *)((int)piVar3 + uVar8 + 4) & 3) != 0) {
      for (uVar13 = 0; uVar13 < *puVar12; uVar13 = uVar13 + 1) {
        local_54 = (uint)*(ushort *)((int)puVar12 + uVar13 * 2 + 4);
        iVar6 = local_54 - param_3;
        uVar5 = local_54;
        if ((int)param_3 <= (int)local_54) goto LAB_00003208;
        if (*(byte *)(param_2 + local_54) == uVar8) {
          if ((*(byte *)((int)piVar3 + uVar8 + 4) & 1) != 0) {
            local_58 = (uint)*(byte *)(iVar9 + 5);
            uVar5 = (uint)*(byte *)(param_2 + local_54 + 1);
            iVar6 = fw_mem_equal(iVar9 + 6,local_58,param_2 + local_54 + 2);
            if (iVar6 == 0) {
              iVar2 = param_1 * 0x3b0 + DAT_00003478;
              if (uVar8 != 0x3d) {
                return 1;
              }
              param_2 = param_2 + local_54;
              iVar10 = fw_mem_equal(iVar10 + uVar4 + 9,local_58 - 3,param_2 + 5,uVar5 - 3);
              if (((*(char *)(iVar9 + 6) == *(char *)(param_2 + 2)) &&
                  ((*(byte *)(iVar9 + 7) & 0xf7) == (*(byte *)(param_2 + 3) & 0xf7))) &&
                 ((uint)(*(byte *)(iVar9 + 8) >> 1) * 2 + (uint)(*(byte *)(param_2 + 4) >> 1) * -2
                  == 0)) {
                bVar14 = true;
              }
              else {
                bVar14 = false;
              }
              bVar15 = false;
              if (((local_58 == uVar5) && (iVar10 != 0)) && (bVar14)) {
                bVar15 = true;
              }
              if (!bVar15) {
                return 1;
              }
              if (*(char *)(iVar2 + 0x33) == '\0') {
                return 1;
              }
              return 0;
            }
          }
          if ((int)((uint)*(byte *)((int)piVar3 + uVar8 + 4) << 0x1e) < 0) goto LAB_00003346;
        }
      }
      bVar1 = *(byte *)((int)piVar3 + uVar8 + 4);
      goto LAB_00003342;
    }
LAB_00003346:
  }
  uVar4 = 0;
  do {
    if (*puVar12 <= uVar4) {
      return 0;
    }
    uVar13 = (uint)*(ushort *)((int)puVar12 + uVar4 * 2 + 4);
    uVar8 = (uint)*(byte *)(param_2 + uVar13);
    if (((int)((uint)*(byte *)((int)piVar3 + uVar8 + 4) << 0x1d) < 0) || (uVar8 == 0xdd)) {
      uVar5 = 0;
      while( true ) {
        if (*local_48 <= uVar5) {
          return 1;
        }
        uVar7 = (uint)*(ushort *)((int)local_48 + uVar5 * 2 + 4);
        if ((int)param_3 <= (int)uVar7) {
          return 0;
        }
        if (uVar8 == 0xdd) break;
        if (*(byte *)(iVar10 + uVar7 + 4) == uVar8) goto LAB_000033d8;
        uVar5 = uVar5 + 1;
      }
      iVar9 = ie_find_vendor_match(iVar10 + uVar7 + 4,*(undefined4 *)(iVar2 + 0x14c));
      if (((iVar9 == 0) &&
          (iVar9 = ie_find_vendor_match
                             (param_2 + uVar13,*(undefined4 *)(iVar2 + 0x14c),iVar2 + 0x150),
          iVar9 != 0)) && ((int)((uint)*(byte *)(iVar9 + 1) << 0x1d) < 0)) {
        return 1;
      }
    }
LAB_000033d8:
    uVar4 = uVar4 + 1;
  } while( true );
}



/* ======================================================================
 * 000033e6  beacon_filter_check_and_store
 * ====================================================================== */

/* beacon_filter_check_and_store(if_id, beacon, len) -- decide whether a received
   beacon is "interesting" enough to pass to the host.  Returns 1 = deliver,
   0 = suppress.
   
   *** THIS IS THE MIB 0x1004 / 0x1005 BEACON FILTER IMPLEMENTATION. ***
   
     if (!filter_enabled) {                      /* g_bf[if_id].enable == 0 */
         if (period) {                           /* +0x35C: deliver 1-in-N */
             if (++count < period) return 0;
             count = 0;
         }
         return 1;                               /* no filtering */
     }
     ie_index_build(if_id, beacon, len);         /* index all IE offsets */
     if (stored beacon is empty) { store; return 1; }
     changed = beacon_filter_ie_diff(if_id, beacon, len);
     if (changed) { store the new beacon; return 1; }
     return 0;
   
   `beacon_filter_ie_diff` (`0x00003156`) walks the IE index and applies a per-IE-id
   **policy byte table** at `g_bf + if_id*0x188 + 0x48`, indexed by element id.  The
   bits map exactly onto cw1200's constants:
   
     bit 0 (0x01)  WSM_BEACON_FILTER_IE_HAS_CHANGED
                   -> compare the IE body; any difference is significant
     bit 1 (0x02)  WSM_BEACON_FILTER_IE_NO_LONGER_PRESENT
                   -> the IE disappearing is significant
     bit 2 (0x04)  WSM_BEACON_FILTER_IE_HAS_APPEARED
                   -> the IE appearing is significant
   
   That is `struct wsm_beacon_filter_table_entry { u8 ie_id; u8 actions; }` from
   MIB 0x1004, and the enable/period pair at +0x358/+0x35C is MIB 0x1005
   `struct wsm_beacon_filter_control { int enabled; int bcn_count; }`.
   
   Two special cases worth knowing:
   
   * **Vendor IEs (0xDD) are matched by OUI+type**, not just by id, through
     `ie_find_vendor_match` (`0x00003104`) against an 8-byte-entry table at
     `+0x148`/`+0x150`.  So a vendor IE only counts as "changed" if its OUI and type
     match a configured entry.
   * **IE 0x3D (HT Operation) gets a masked comparison**: byte 3 is compared with
     `& 0xF7` and byte 4 only on `>> 1`, i.e. two flag bits are deliberately ignored
     so that routine HT-protection churn does not wake the host.  There is also a
     per-vif override at `vif+0x33` that forces HT changes to be significant.
   
   The stored-beacon buffers are double-buffered (`+0x2C4` / `+0x4C8`, selected by
   the flag at `+0x2C0`), each capped at **700 bytes** — the same cap as
   template_replace_ie, and beacons longer than 699 skip filtering entirely. */

int beacon_filter_check_and_store(int param_1,void *param_2,uint param_3)

{
  byte bVar1;
  int iVar2;
  uint uVar3;
  uint *puVar4;
  uint uVar5;
  uint uVar6;
  uint *puVar7;
  uint uVar8;
  int iVar9;
  int *piVar10;
  uint uVar11;
  bool bVar12;
  uint local_30;
  int local_28;
  
  local_28 = 0;
  bVar12 = param_2 == (void *)0x0;
  do {
    while( true ) {
      if (bVar12) {
        return 0;
      }
      iVar9 = param_1 * 0x188 + DAT_00003474;
      iVar2 = param_1 * 0xc + DAT_00003474;
      piVar10 = (int *)(iVar9 + 0x48);
      puVar4 = (uint *)(DAT_0000346c * param_1 + DAT_00003470);
      if (*(int *)(iVar2 + 0x358) != 0) break;
      bVar12 = true;
      if (*(int *)(iVar2 + 0x35c) != 0) {
        uVar5 = *(int *)(iVar2 + 0x360) + 1;
        *(uint *)(iVar2 + 0x360) = uVar5;
        if (uVar5 < *(uint *)(iVar2 + 0x35c)) {
          return 0;
        }
        *(undefined4 *)(iVar2 + 0x360) = 0;
        goto LAB_0000360c;
      }
    }
    iVar2 = ie_index_build(param_1,param_2,param_3);
    puVar7 = (uint *)(DAT_0000346c * param_1 + DAT_00003470 + 0x2c0);
    if (iVar2 == 0) {
      if (*puVar4 == 0) {
        if ((int)param_3 < 0x2bd) {
          *puVar4 = param_3;
          fw_memcpy(puVar4 + 1,param_2,param_3);
          *puVar7 = (uint)(*puVar7 == 0);
        }
        goto LAB_0000360c;
      }
      if ((param_3 < 0x25) || (699 < (int)param_3)) goto LAB_0000360c;
      local_28 = beacon_filter_ie_diff(param_1,param_2,param_3);
      goto LAB_000035e0;
    }
    bVar12 = true;
  } while (*piVar10 == 0);
  local_30 = 0;
  uVar5 = *puVar7;
  do {
    if (puVar4[uVar5 * 0x81 + 0xb1] <= local_30) {
LAB_000035e0:
      if (local_28 == 1) {
LAB_000035e6:
        *puVar4 = param_3;
        fw_memcpy(puVar4 + 1,param_2,param_3);
        puVar4[0xb0] = (uint)(puVar4[0xb0] == 0);
LAB_0000360c:
        local_28 = 1;
      }
      return local_28;
    }
    uVar8 = (uint)*(ushort *)((int)(puVar4 + uVar5 * 0x81 + 0xb1) + local_30 * 2 + 4);
    uVar11 = (uint)*(byte *)((int)puVar4 + uVar8 + 4);
    if (*(byte *)((int)param_2 + uVar8) == uVar11) {
      if (uVar11 == 0xdd) {
        iVar2 = ie_find_vendor_match((int)puVar4 + uVar8 + 4,*(undefined4 *)(iVar9 + 0x14c));
        if ((iVar2 == 0) || ((*(byte *)(iVar2 + 1) & 3) == 0)) {
          iVar2 = ie_find_vendor_match
                            ((int)param_2 + uVar8,*(undefined4 *)(iVar9 + 0x14c),iVar9 + 0x150);
          if (iVar2 != 0) {
            bVar1 = *(byte *)(iVar2 + 1);
            goto LAB_000035ca;
          }
        }
        else {
          iVar2 = fw_mem_equal((int)puVar4 + uVar8 + 6,*(undefined1 *)((int)puVar4 + uVar8 + 5),
                               (int)param_2 + uVar8 + 2,*(undefined1 *)((int)param_2 + uVar8 + 1));
          if (iVar2 == 0) goto LAB_000035e6;
        }
      }
      else if ((*(byte *)((int)piVar10 + uVar11 + 4) & 1) != 0) {
        uVar3 = (uint)*(byte *)((int)puVar4 + uVar8 + 5);
        uVar6 = (uint)*(byte *)((int)param_2 + uVar8 + 1);
        iVar2 = fw_mem_equal((int)puVar4 + uVar8 + 6,uVar3,(int)param_2 + uVar8 + 2);
        if (iVar2 == 0) {
          iVar2 = param_1 * 0x3b0 + DAT_00003618;
          if (uVar11 != 0x3d) goto LAB_000035e6;
          iVar9 = fw_mem_equal((int)puVar4 + uVar8 + 9,uVar3 - 3,(int)param_2 + uVar8 + 5,uVar6 - 3)
          ;
          bVar12 = false;
          if (((uVar3 == uVar6) && (iVar9 != 0)) &&
             (*(char *)((int)puVar4 + uVar8 + 6) == *(char *)((int)param_2 + uVar8 + 2))) {
            bVar12 = true;
          }
          if ((!bVar12) || (*(char *)(iVar2 + 0x33) == '\0')) goto LAB_000035e6;
          goto LAB_000035e0;
        }
      }
    }
    else {
      if ((int)((uint)*(byte *)((int)piVar10 + uVar11 + 4) << 0x1e) < 0) goto LAB_000035e6;
      bVar1 = *(byte *)((int)piVar10 + *(byte *)((int)param_2 + uVar8) + 4);
LAB_000035ca:
      if ((int)((uint)bVar1 << 0x1d) < 0) goto LAB_000035e6;
    }
    local_30 = local_30 + 1;
  } while( true );
}



/* ======================================================================
 * 0000361c  p2p_build_action_frame
 * ====================================================================== */

void p2p_build_action_frame(int param_1,void *param_2,ushort *param_3,void *param_4,uint param_5)

{
  ushort *puVar1;
  ushort *puVar2;
  ushort uVar3;
  int iVar4;
  int iVar5;
  
  iVar4 = param_1 * 0x40 + DAT_000039e4;
  if (*(short *)(iVar4 + 0x32) == 0) {
    fw_assert(s_wsmlmac_c_000039e8,0x126,0xe);
  }
  puVar1 = *(ushort **)(iVar4 + 0x34);
  uVar3 = *puVar1;
  iVar4 = (uint)*(byte *)(DAT_000039f4 + (param_5 >> 1)) + ((int)~((uint)uVar3 << 0x18) >> 0x1f) * 2
  ;
  iVar5 = param_1 * 0x98 + DAT_000039f8;
  puVar2 = (ushort *)(iVar5 + DAT_000039fc);
  if (*(int *)(param_1 * 0x3b0 + DAT_00003a00 + 0x1c) * 0x20000000 < 0) {
    *puVar1 = uVar3 & 0xfeff | 0x200;
    puVar1[2] = *param_3;
    puVar1[3] = param_3[1];
    puVar1[4] = param_3[2];
    puVar1[5] = *(ushort *)(iVar5 + 0x482);
    puVar1[6] = *(ushort *)(iVar5 + 0x484);
    puVar1[7] = *(ushort *)(iVar5 + 0x486);
    puVar1[8] = *puVar2;
    puVar1[9] = puVar2[1];
    uVar3 = puVar2[2];
  }
  else {
    *puVar1 = uVar3 & 0xfdff | 0x100;
    puVar1[2] = *(ushort *)(iVar5 + 0x482);
    puVar1[3] = *(ushort *)(iVar5 + 0x484);
    puVar1[4] = *(ushort *)(iVar5 + 0x486);
    puVar1[5] = *puVar2;
    puVar1[6] = puVar2[1];
    puVar1[7] = puVar2[2];
    puVar1[8] = *param_3;
    puVar1[9] = param_3[1];
    uVar3 = param_3[2];
  }
  puVar1[10] = uVar3;
  *(ushort *)((int)puVar1 + iVar4 + 0x34) = *param_3;
  *(ushort *)((int)puVar1 + iVar4 + 0x36) = param_3[1];
  *(ushort *)((int)puVar1 + iVar4 + 0x38) = param_3[2];
  *(ushort *)((int)puVar1 + iVar4 + 0x2a) = *puVar2;
  *(ushort *)((int)puVar1 + iVar4 + 0x2c) = puVar2[1];
  *(ushort *)((int)puVar1 + iVar4 + 0x2e) = puVar2[2];
  fw_memcpy((void *)((int)puVar1 + iVar4 + 0x3a),param_4,4);
  fw_memcpy((void *)((int)puVar1 + iVar4 + 0x30),param_2,4);
  tx_send_template_frame(param_1,6);
  return;
}



/* ======================================================================
 * 0000371c  ind_0805_event_a
 * ====================================================================== */

void ind_0805_event_a(short param_1,int *param_2)

{
  int iVar1;
  int iVar2;
  ushort uVar3;
  ushort *puVar4;
  uint uVar5;
  void *src;
  int iVar6;
  int iVar7;
  int iVar8;
  uint local_1c;
  undefined4 local_18;
  
  puVar4 = (ushort *)hif_alloc_msg_to_host(0xc);
  if (puVar4 == (ushort *)0x0) {
    iVar8 = *param_2;
    if (((iVar8 != 4) && (iVar8 != 7)) && (iVar8 != 0)) {
      fw_assert(s_wsmlmac_c_000039e8,0x840,5);
      return;
    }
  }
  else {
    puVar4[1] = param_1 << 6 | (ushort)DAT_00003a04;
    *puVar4 = 0xc;
    *(int *)(puVar4 + 2) = *param_2;
    iVar8 = DAT_00003a08;
    if (*param_2 == 4) {
      *(char *)(puVar4 + 4) = (char)param_2[1];
      *(undefined1 *)(iVar8 + 0x13) = 1;
    }
    else {
      *(int *)(puVar4 + 4) = param_2[1];
    }
    iVar8 = DAT_00003a0c;
    if (*param_2 == 0) {
      iVar6 = ((*(uint *)(DAT_00003a0c + 0x10) & 0x3ffffff) >> 0x18) * 0x6c + DAT_00003a10;
      local_1c = (uint)*(byte *)(iVar6 + 0xa2);
      local_18 = DAT_00003a14;
      iVar7 = iVar6 + 0xa0;
      if (*(char *)(iVar6 + 0xa3) == '\0') {
        iVar6 = (uint)*DAT_00003a18 * 0x6c + DAT_00003a10;
        if (*(char *)(iVar6 + 0xa3) != '\0') {
          local_1c = (uint)*(byte *)(iVar6 + 0xa2);
          local_18 = DAT_00003a1c;
          iVar7 = iVar6 + 0xa0;
        }
      }
      if (*puVar4 + 0x28 < 0x181) {
        *(undefined4 *)((int)puVar4 + (uint)*puVar4) = *(undefined4 *)(DAT_00003a20 + 0x34);
        iVar6 = DAT_00003a24;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar6 + 4);
        iVar6 = DAT_00003a28;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar6 + 0x24);
        iVar6 = DAT_00003a2c;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar6 + 0x38);
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar8 + 0x10);
        iVar8 = DAT_00003a30;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar8 + 0x20);
        iVar8 = DAT_00003a30;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar8 + 0xa0);
        iVar1 = DAT_00003a34;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar1 + 0x20);
        iVar2 = DAT_00003a34;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(undefined4 *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) = *(undefined4 *)(iVar2 + 0xa0);
        iVar6 = DAT_00003a30;
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        *(byte *)((int)puVar4 + (uint)(ushort)(uVar3 + 4)) =
             (byte)*(undefined4 *)(iVar6 + 0x14) | (byte)(*(int *)(iVar8 + 0x94) << 1) |
             (byte)(*(int *)(iVar1 + 0x14) << 2) | (byte)(*(int *)(iVar2 + 0x94) << 3);
        *puVar4 = *puVar4 + 4;
      }
      uVar5 = 0;
      do {
        if (*puVar4 + 8 < 0x181) {
          fw_memcpy((void *)((uint)*puVar4 + (int)puVar4),
                    (void *)(uVar5 * 0x6c + DAT_00003a10 + 0xa0),8);
          *puVar4 = *puVar4 + 8;
        }
        uVar5 = uVar5 + 1 & 0xff;
      } while (uVar5 < 4);
      if (*puVar4 + 0x10 < 0x181) {
        fw_memcpy((void *)((uint)*puVar4 + (int)puVar4),(void *)(iVar7 + 0xc),4);
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        fw_memcpy((void *)((uint)(ushort)(uVar3 + 4) + (int)puVar4),(void *)(iVar7 + 0x24),4);
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        fw_memcpy((void *)((uint)(ushort)(uVar3 + 4) + (int)puVar4),(void *)(iVar7 + 0x3c),4);
        uVar3 = *puVar4;
        *puVar4 = uVar3 + 4;
        fw_memcpy((void *)((uint)(ushort)(uVar3 + 4) + (int)puVar4),(void *)(iVar7 + 0x54),4);
        *puVar4 = *puVar4 + 4;
      }
      while( true ) {
        iVar8 = local_1c * 0x18 + iVar7;
        src = *(void **)(iVar8 + 0x18);
        if (0x180 < *puVar4 + 4) break;
        *(undefined4 *)((int)puVar4 + (uint)*puVar4) = local_18;
        uVar3 = *puVar4 + 4;
        *puVar4 = uVar3;
        if (src != (void *)0x0) {
          if (*(char *)(iVar8 + 0xc) == '\x01') {
            do {
              if (*puVar4 + 0x84 < 0x181) {
                fw_memcpy((void *)((uint)*puVar4 + (int)puVar4),src,0x30);
                uVar3 = *puVar4;
                *puVar4 = uVar3 + 0x30;
                fw_memcpy((void *)((uint)(ushort)(uVar3 + 0x30) + (int)puVar4),
                          *(void **)(iVar8 + 0x20),0x54);
                *puVar4 = *puVar4 + 0x54;
              }
              src = *(void **)((int)src + 0x3c);
            } while (src != (void *)0x0);
          }
          else if (uVar3 + 0x84 < 0x181) {
            fw_memcpy((void *)((uint)uVar3 + (int)puVar4),src,0x30);
            uVar3 = *puVar4;
            *puVar4 = uVar3 + 0x30;
            fw_memcpy((void *)((uint)(ushort)(uVar3 + 0x30) + (int)puVar4),*(void **)(iVar8 + 0x20),
                      0x54);
            *puVar4 = *puVar4 + 0x54;
          }
        }
        if (*(byte *)(iVar7 + 1) == local_1c) break;
        local_1c = local_1c + 1 & 3;
      }
    }
    hif_send_msg_to_host(puVar4);
  }
  return;
}



/* ======================================================================
 * 000039b2  ind_080c_suspend_resume
 * ====================================================================== */

void ind_080c_suspend_resume(short param_1,undefined2 *param_2)

{
  undefined2 *puVar1;
  
  puVar1 = (undefined2 *)hif_alloc_msg_to_host(0xc);
  if (puVar1 != (undefined2 *)0x0) {
    puVar1[1] = param_1 << 6 | (short)DAT_00003a04 + 7U;
    *puVar1 = 0xc;
    puVar1[2] = *param_2;
    puVar1[3] = 0;
    puVar1[4] = 0;
    puVar1[5] = 0;
    hif_send_msg_to_host();
  }
  return;
}



/* ======================================================================
 * 00003a38  ie_find
 * ====================================================================== */

byte * ie_find(byte *buf,int len,uint eid,int skip)

{
  byte *pbVar1;
  
  if (eid == 0xc) {
    len = len + -4;
  }
  pbVar1 = buf + len;
  do {
    if (pbVar1 <= buf) {
      return (byte *)0x0;
    }
    if (*buf == eid) {
      if (skip == 0) {
        if (pbVar1 < buf + buf[1] + 2) {
          return (byte *)0x0;
        }
        return buf;
      }
      skip = skip + -1;
    }
    buf = buf + buf[1] + 2;
  } while( true );
}



/* ======================================================================
 * 00003a66  tlv_find_u16len
 * ====================================================================== */

byte * tlv_find_u16len(byte *param_1,int param_2,uint param_3)

{
  byte *pbVar1;
  int iVar2;
  
  pbVar1 = param_1 + param_2;
  while( true ) {
    if (pbVar1 <= param_1) {
      return (byte *)0x0;
    }
    iVar2 = (uint)param_1[1] + (uint)param_1[2] * 0x100 + 3;
    if (*param_1 == param_3) break;
    param_1 = param_1 + iVar2;
  }
  if (pbVar1 < param_1 + iVar2) {
    return (byte *)0x0;
  }
  return param_1;
}



/* ======================================================================
 * 00003a8e  ie_find_in_frame
 * ====================================================================== */

byte * ie_find_in_frame(int param_1,int param_2,uint param_3,int param_4)

{
  byte *pbVar1;
  
  pbVar1 = (byte *)(param_1 + 0x24);
  do {
    if ((byte *)(param_1 + param_2) <= pbVar1) {
      return (byte *)0x0;
    }
    if (*pbVar1 == param_3) {
      if ((byte *)(param_1 + param_2) < pbVar1 + pbVar1[1] + 2) {
        return (byte *)0x0;
      }
      if (param_4 == 0) {
        return pbVar1;
      }
      param_4 = param_4 + -1;
    }
    pbVar1 = pbVar1 + pbVar1[1] + 2;
  } while( true );
}



/* ======================================================================
 * 00003ac0  ie_find_in_mgmt_frame
 * ====================================================================== */

ushort * ie_find_in_mgmt_frame(ushort *param_1,int param_2,uint param_3,int param_4)

{
  char cVar1;
  ushort *puVar2;
  ushort uVar3;
  ushort *puVar4;
  
  puVar4 = param_1 + 0xd;
  uVar3 = *param_1 & 0xff;
  puVar2 = (ushort *)((int)param_1 + param_2);
  if (((((uVar3 != 0xa0) && (uVar3 != 0xc0)) && (puVar4 = puVar2, uVar3 == 0xd0)) &&
      ((cVar1 = (char)param_1[0xc], cVar1 != '\0' && (cVar1 != '\x01')))) && (cVar1 == '\x03')) {
    cVar1 = *(char *)((int)param_1 + 0x19);
    puVar4 = (ushort *)((int)param_1 + 0x21);
    if (((cVar1 != '\0') && (cVar1 != '\x01')) && (puVar4 = puVar2, cVar1 == '\x02')) {
      puVar4 = param_1 + 0xf;
    }
  }
  do {
    if (puVar2 <= puVar4) {
      return (ushort *)0x0;
    }
    if ((byte)*puVar4 == param_3) {
      if (puVar2 < (ushort *)((int)puVar4 + *(byte *)((int)puVar4 + 1) + 2)) {
        return (ushort *)0x0;
      }
      if (param_4 == 0) {
        return puVar4;
      }
      param_4 = param_4 + -1;
    }
    puVar4 = (ushort *)((int)puVar4 + *(byte *)((int)puVar4 + 1) + 2);
  } while( true );
}



/* ======================================================================
 * 00003b28  frame_is_unprotected_mgmt
 * ====================================================================== */

undefined4 frame_is_unprotected_mgmt(ushort *param_1)

{
  char cVar1;
  uint uVar2;
  
  uVar2 = *param_1 & 0xff;
  if (((uVar2 != 0xa0) && (uVar2 != 0xc0)) &&
     ((uVar2 != 0xd0 ||
      ((-1 < (int)((uint)*param_1 << 0x11) &&
       (((cVar1 = (char)param_1[0xc], cVar1 == '\x04' || (cVar1 == '\a')) || (cVar1 == '\x7f')))))))
     ) {
    return 0;
  }
  return 1;
}



/* ======================================================================
 * 00003b54  ie_find_p2p_vendor
 * ====================================================================== */

byte * ie_find_p2p_vendor(byte *param_1,uint param_2)

{
  uint uVar1;
  byte *pbVar2;
  byte *pbVar3;
  
  uVar1 = DAT_00003ba8;
  while (((pbVar3 = (byte *)0x0, param_2 != 0 &&
          (pbVar2 = ie_find(param_1,param_2,0xdd,0), pbVar2 != (byte *)0x0)) &&
         (((uint)pbVar2[2] != (uVar1 & 0xff) || (pbVar3 = pbVar2, pbVar2[5] != 9))))) {
    if (param_2 < ((uint)(pbVar2 + ((uint)pbVar2[1] - (int)param_1)) & 0xffff)) {
      return (byte *)0x0;
    }
    param_2 = param_2 - ((uint)(pbVar2 + ((uint)pbVar2[1] - (int)param_1)) & 0xffff) & 0xffff;
    param_1 = pbVar2 + pbVar2[1] + 2;
  }
  return pbVar3;
}



/* ======================================================================
 * 00003d38  tx_send_null_data
 * ====================================================================== */

void tx_send_null_data(uint param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  ushort uVar1;
  undefined1 uVar2;
  int iVar3;
  int iVar4;
  char *pcVar5;
  ushort *dst;
  
  if (param_2 == 4) {
    *(undefined2 *)(param_1 * 0x104 + DAT_00003f9c + 0x138) = 0xfe;
  }
  if (((*(byte *)(DAT_00003fac + 10) & 7) != 0) && (param_2 == 4)) {
LAB_00003de2:
    *(byte *)(DAT_00003fac + 0xb) =
         *(byte *)(DAT_00003fac + 0xb) | (byte)(2 << ((param_1 & 0x3f) << 2));
    return;
  }
  iVar3 = tx_ctx_alloc_init(param_2,3,0,param_4,param_4);
  if (iVar3 == 0) {
    if (param_2 == 4) goto LAB_00003de2;
  }
  else {
    iVar4 = param_1 * 0x3b0 + DAT_00003f98;
    pcVar5 = (char *)(iVar4 + 0x18);
    dst = *(ushort **)(iVar3 + 0x1c);
    fw_memcpy(dst,(void *)(iVar4 + 0x13c),0x18);
    *dst = *dst | 0x48;
    *(undefined2 *)(iVar3 + 0x5c) = 0x18;
    if (*pcVar5 == '\x02') {
      uVar1 = (ushort)DAT_00003fc0;
      dst[2] = uVar1;
      dst[3] = uVar1;
      dst[4] = uVar1;
    }
    uVar2 = link_lookup_by_mac(param_1,dst + 2);
    *(undefined1 *)(iVar3 + 0xbf) = uVar2;
    *(char *)(iVar3 + 0xbd) = (char)param_1;
    *(undefined1 *)(iVar3 + 0xc) = *(undefined1 *)(param_1 * 0x40 + DAT_00003fc4 + 0x11);
    if (param_2 == 4) {
      *(byte *)(DAT_00003fac + 10) = *(byte *)(DAT_00003fac + 10) | 2;
    }
    lmc_tx_assign_default_rate(iVar3);
  }
  return;
}



/* ======================================================================
 * 00003dec  ps_keepalive_tick
 * ====================================================================== */

void ps_keepalive_tick(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined2 uVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  undefined4 local_1c;
  undefined4 local_18;
  
  uVar4 = 0;
  local_1c = param_3;
  local_18 = param_4;
  do {
    iVar3 = (param_1 << 0x1f) >> 0x1f;
    param_1 = -iVar3;
    iVar3 = iVar3 * -0x3b0 + DAT_00003f98;
    if ((*(uint *)(iVar3 + 0x1c) & 0x101) == 0x101) {
      if (*(ushort *)(iVar3 + 0x1d0) == 0) {
LAB_00003e60:
        if (*(ushort *)(iVar3 + 0x1e0) != 0) {
          if (*(ushort *)(iVar3 + 0x1e2) < *(ushort *)(iVar3 + 0x1e0)) {
            *(ushort *)(iVar3 + 0x1e2) = *(ushort *)(iVar3 + 0x1e2) + 1;
          }
          if ((*(ushort *)(iVar3 + 0x1e0) <= *(ushort *)(iVar3 + 0x1e2)) &&
             (-1 < *DAT_00003fc8 << 0x1a)) {
            *(undefined2 *)(iVar3 + 0x1e2) = 0;
            tx_send_null_data(param_1,2);
            return;
          }
        }
      }
      else {
        if (*(ushort *)(iVar3 + 0x1d2) < *(ushort *)(iVar3 + 0x1d0)) {
          *(ushort *)(iVar3 + 0x1d2) = *(ushort *)(iVar3 + 0x1d2) + 1;
        }
        if (*(ushort *)(iVar3 + 0x1d0) <= *(ushort *)(iVar3 + 0x1d2)) {
          *(undefined2 *)(iVar3 + 0x1d2) = 0;
          uVar1 = (undefined2)DAT_00003fc0;
          local_1c = CONCAT22(uVar1,uVar1);
          local_18 = CONCAT22(local_18._2_2_,uVar1);
          iVar2 = p2p_build_action_frame
                            (param_1,iVar3 + 0x1d4,&local_1c,iVar3 + 0x1d8,
                             *(undefined1 *)(iVar3 + 0x1dc));
          if (iVar2 != 0) {
            *(undefined2 *)(iVar3 + 0x1e2) = 0;
            return;
          }
          if (*(short *)(iVar3 + 0x1d0) == 0) goto LAB_00003e60;
        }
      }
    }
    param_1 = param_1 + 1;
    uVar4 = uVar4 + 1;
    if (1 < uVar4) {
      return;
    }
  } while( true );
}



/* ======================================================================
 * 00003ea6  rx_beacon_check_tim_for_us
 * ====================================================================== */

uint rx_beacon_check_tim_for_us(int param_1,int param_2)

{
  byte bVar1;
  char cVar2;
  byte bVar3;
  uint uVar4;
  uint uVar5;
  int iVar6;
  int iVar7;
  
  iVar6 = 0;
  uVar4 = 0;
  if ((param_2 != 0) && (3 < *(byte *)(param_2 + 1))) {
    iVar7 = param_1 * 0x3b0 + DAT_00003f98;
    if (*DAT_00003fcc == 0) {
      *DAT_00003fcc = 1;
      *(undefined1 *)(iVar7 + 0x110) = *(undefined1 *)(param_2 + 3);
    }
    bVar1 = *(byte *)(param_2 + 1);
    cVar2 = *(char *)(param_2 + 2);
    bVar3 = *(byte *)(param_2 + 4);
    *(char *)(DAT_00003fd0 + 8) = cVar2;
    if (((bVar3 & 1) != 0) && (cVar2 == '\0')) {
      iVar6 = 1;
    }
    uVar5 = *(int *)(iVar7 + 0x3bc) + (uint)(bVar3 >> 1) * -0x10;
    if (-1 < (int)uVar5) {
      if ((uVar5 >> 3 < bVar1 - 3) &&
         (((uint)*(byte *)(param_2 + (uVar5 >> 3) + 5) & 1 << (uVar5 & 7)) != 0)) {
        uVar4 = 1;
      }
    }
  }
  return iVar6 << 8 | uVar4;
}



/* ======================================================================
 * 00003f20  beacon_rx_post_process
 * ====================================================================== */

void beacon_rx_post_process(int param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  
  if (*(int *)(DAT_00003fd4 + 0x28) != 0) {
    *(undefined2 *)(param_1 * 0x70 + DAT_00003fd8 + 0x2c) = 0xfe;
    iVar2 = DAT_00003fdc;
    iVar1 = DAT_00003f98;
    if (*(short *)(DAT_00003f84 + 0x12) != 0) {
      uVar4 = 0;
      *(undefined1 *)(DAT_00003f98 + 8) = 0;
      do {
        iVar3 = uVar4 * 8 + iVar1 + iVar2;
        if ((*(byte *)(iVar3 + 6) & 1) != 0) {
          *(byte *)(iVar3 + 6) = *(byte *)(iVar3 + 6) & 0xf7;
          if ((int)*(char *)(iVar3 + 7) + 0x18U < 0x19) {
            *(char *)(iVar3 + 7) = *(char *)(iVar3 + 7) + -1;
          }
        }
        uVar4 = uVar4 + 1;
      } while (uVar4 < 8);
    }
  }
  beacon_set_tim_bcast_bit();
  beacon_dtim_countdown();
  return;
}



/* ======================================================================
 * 00003fe0  tx_send_qos_null
 * ====================================================================== */

void tx_send_qos_null(uint param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  undefined1 uVar2;
  ushort uVar3;
  int iVar4;
  int iVar5;
  ushort *dst;
  
  iVar1 = DAT_0000433c;
  if ((*(byte *)(DAT_0000433c + 10) & 7) == 0) {
    iVar5 = param_1 * 0x3b0 + DAT_00004340;
    if (-1 < *(int *)(iVar5 + 0x1c) << 2) {
      iVar4 = tx_ctx_alloc_init(8,*(undefined1 *)(DAT_00004344 + param_2),1,param_4,iVar5,param_1);
      if (iVar4 == 0) goto LAB_0000406e;
      dst = *(ushort **)(iVar4 + 0x1c);
      fw_memcpy(dst,(void *)(iVar5 + 0x13c),0x18);
      *dst = *dst | 200;
      uVar3 = qos_ac_to_param(param_2);
      dst[0xc] = uVar3 & 0xf;
      *(undefined2 *)(iVar4 + 0x5c) = 0x1a;
      uVar2 = link_lookup_by_mac(param_1,dst + 2);
      *(undefined1 *)(iVar4 + 0xbf) = uVar2;
      *(char *)(iVar4 + 0xbd) = (char)param_1;
      *(undefined1 *)(iVar4 + 0xc) = *(undefined1 *)(param_1 * 0x40 + DAT_00004348 + 0x19);
      *(byte *)(iVar1 + 10) = *(byte *)(iVar1 + 10) | 4;
      lmc_tx_assign_default_rate(iVar4);
    }
    return;
  }
LAB_0000406e:
  *(byte *)(iVar1 + 0xb) = *(byte *)(iVar1 + 0xb) | (byte)(4 << ((param_1 & 0x3f) << 2));
  return;
}



/* ======================================================================
 * 00004076  tx_send_ps_poll
 * ====================================================================== */

void tx_send_ps_poll(uint param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  ushort *dst;
  int iVar3;
  
  if ((*(byte *)(DAT_0000433c + 10) & 7) == 0) {
    iVar3 = param_1 * 0x3b0 + DAT_00004340;
    if (*(int *)(iVar3 + 0x1c) << 2 < 0) {
      return;
    }
    iVar2 = tx_ctx_alloc_init(7,0,1,param_4,param_4);
    if (iVar2 != 0) {
      dst = *(ushort **)(iVar2 + 0x1c);
      fw_memcpy(dst,(void *)(iVar3 + 0x13c),0x12);
      *dst = *dst | 0xa4;
      dst[1] = (ushort)*(undefined4 *)(iVar3 + 0x3bc) | 0xc000;
      *(undefined2 *)(iVar2 + 0x5c) = 0x10;
      *(uint *)(iVar2 + 0x58) = *(uint *)(iVar2 + 0x58) | 0x2000000;
      *(undefined1 *)(iVar2 + 0xbf) = 0xf;
      iVar1 = DAT_00004348;
      *(char *)(iVar2 + 0xbd) = (char)param_1;
      iVar3 = DAT_0000433c;
      *(undefined1 *)(iVar2 + 0xc) = *(undefined1 *)(param_1 * 0x40 + iVar1 + 0x21);
      *(byte *)(iVar3 + 10) = *(byte *)(iVar3 + 10) | 1;
      lmc_tx_assign_default_rate(iVar2);
      return;
    }
  }
  else if ((*(byte *)(DAT_0000433c + 10) & 1) != 0) {
    return;
  }
  *(byte *)(DAT_0000433c + 0xb) =
       *(byte *)(DAT_0000433c + 0xb) | (byte)(1 << ((param_1 & 0x3f) << 2));
  return;
}



/* ======================================================================
 * 0000411a  tx_send_template_frame
 * ====================================================================== */

undefined4 tx_send_template_frame(int param_1,int param_2)

{
  ushort uVar1;
  undefined1 uVar2;
  xr_tx_ctx *ctx;
  undefined4 uVar3;
  int iVar4;
  int iVar5;
  byte bVar6;
  void *dst;
  int iVar7;
  
  ctx = (xr_tx_ctx *)tx_ctx_alloc_init(2,0,1);
  iVar4 = DAT_00004348;
  if (ctx == (xr_tx_ctx *)0x0) {
    uVar3 = 0;
  }
  else {
    ctx->field_0xbd = (char)param_1;
    iVar4 = param_1 * 0x40 + iVar4;
    if (param_2 == 7) {
      iVar4 = iVar4 + 0x38;
    }
    else {
      iVar4 = iVar4 + 0x30;
    }
    dst = (void *)ctx->dwHdr80211;
    bVar6 = *(byte *)(iVar4 + 1);
    if (bVar6 == 0xff) {
      iVar5 = param_1 * 0x3b0 + DAT_00004340;
      iVar7 = param_1 * 0x3b0 + DAT_00004340;
      if (*(char *)(iVar5 + 0x26) == '\0') {
        bVar6 = *(byte *)(iVar7 + 300);
      }
      else {
        bVar6 = *(byte *)(iVar7 + 0x12d);
      }
      if ((bVar6 == 0xff) || (*(byte *)(iVar5 + 0x27) < 3)) {
        bVar6 = *(byte *)(iVar5 + 0x24);
      }
    }
    uVar3 = DAT_0000434c;
    if ((int)((uint)bVar6 << 0x18) < 0) {
      bVar6 = bVar6 & 0x7f;
      uVar3 = 0x40800000;
    }
    *(undefined4 *)&ctx->field_0x58 = uVar3;
    ctx->field_0xc = bVar6;
    ctx->field_0x63 = bVar6;
    ctx->field_0x24 = bVar6;
    ctx->bAllocFlags = 8;
    *(undefined4 *)ctx = 0;
    *(undefined4 *)&ctx->field_0xd8 = 0;
    *(undefined4 *)&ctx->field_0x130 = 0;
    uVar1 = *(ushort *)(iVar4 + 2);
    *(ushort *)&ctx->field_0x5c = uVar1;
    *(undefined4 *)&ctx->field_0x14 = 0;
    *(uint *)&ctx->field_0x18 = (uint)uVar1;
    iVar5 = DAT_00004350;
    ctx->field_0xe = 0;
    *(undefined4 *)&ctx->field_0x98 = *(undefined4 *)(iVar5 + 0x44);
    *(undefined4 *)&ctx->field_0x90 = 0;
    ctx->field_0x60 = *(undefined1 *)(DAT_00004354 + (uint)ctx->bQueueId);
    *(void **)&ctx->field_0x54 = dst;
    *(undefined4 *)&ctx->field_0x64 = 0;
    txq_set_frame_lifetime(&ctx->field_0x54,0);
    ctx->field_0xab = 0xff;
    ctx->field_0x62 = (char)((ctx->bAllocFlags & 0x7f) >> 4);
    pas_tx_policy_prepare(&ctx->field_0x54);
    fw_memcpy(dst,*(void **)(iVar4 + 4),(uint)*(ushort *)(iVar4 + 2));
    uVar2 = link_lookup_by_mac(param_1,(int)dst + 4);
    ctx->field_0xbf = uVar2;
    tx_classify_hdr_len(ctx);
    *(uint *)&ctx->field_0x80 = *(uint *)&ctx->field_0x80 | 2;
    mic_build_aad_and_submit(ctx);
    uVar3 = 1;
  }
  return uVar3;
}



/* ======================================================================
 * 00004238  event_send_rcpi_rssi
 * ====================================================================== */

void event_send_rcpi_rssi(undefined4 param_1,undefined4 param_2)

{
  undefined4 local_10;
  undefined4 local_c;
  
  if (*(char *)(DAT_0000433c + -0xbc) == '\0') {
    local_10 = 4;
    local_c = param_2;
    ind_0805_event_a(param_1,&local_10);
  }
  return;
}



/* ======================================================================
 * 00004252  tx_requeue_after_ps
 * ====================================================================== */

void tx_requeue_after_ps(int param_1,int param_2)

{
  int iVar1;
  
  if ((*(char *)(param_1 + 0xe) != '\x0f') && (param_2 != 0)) {
    iVar1 = (uint)*(byte *)(param_1 + 0x69) * 0x3b0 + DAT_00004340;
    *(uint *)(iVar1 + 0x1c) = *(uint *)(iVar1 + 0x1c) | 0x80000;
  }
  *(undefined1 *)(param_1 + 0x53) = 0;
  pas_txq_push_global();
  return;
}



/* ======================================================================
 * 00004368  rx_beacon_validate_channel
 * ====================================================================== */

undefined4 rx_beacon_validate_channel(int *param_1)

{
  int iVar1;
  char *pcVar2;
  
  pcVar2 = (char *)(*param_1 + 0x24);
  do {
    if (*pcVar2 == '\x03') {
      if ((ushort)(byte)pcVar2[2] != *(ushort *)(DAT_00004548 + 2)) {
        if (*(short *)((int)param_1 + 0x12) == 0x80) {
          *(int *)(DAT_0000454c + 0xc) = *(int *)(DAT_0000454c + 0xc) + 1;
          return 0;
        }
        *(int *)(DAT_0000454c + 0x10) = *(int *)(DAT_0000454c + 0x10) + 1;
        return 0;
      }
      break;
    }
    pcVar2 = pcVar2 + (byte)pcVar2[1] + 2;
  } while (pcVar2 < (char *)(*param_1 + (uint)*(ushort *)(param_1 + 1)));
  if (*(int *)(DAT_00004550 + 0x34) != 0) {
    iVar1 = ie_ssid_matches_list();
    if (iVar1 == 0) {
      return 0;
    }
    param_1[8] = param_1[8] | 0x400;
  }
  return 1;
}



/* ======================================================================
 * 000043ce  rx_beacon_update_erp_ht_flags
 * ====================================================================== */

bool rx_beacon_update_erp_ht_flags(int *param_1)

{
  byte *pbVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  undefined4 local_30 [7];
  
  uVar7 = 0;
  uVar6 = 0;
  iVar2 = ie_find_in_frame(*param_1,(short)param_1[1],0x2a,0);
  if (iVar2 != 0) {
    uVar6 = *(byte *)(iVar2 + 2) & 7;
  }
  iVar2 = ie_peer_supports_ofdm(*param_1 + 0x24,*(ushort *)(param_1 + 1) - 0x24);
  if (iVar2 == 0) {
    uVar6 = uVar6 | 8;
  }
  iVar3 = ie_find_in_frame(*param_1,(short)param_1[1],0x3d,0);
  iVar2 = DAT_0000455c;
  pbVar1 = DAT_00004558;
  if (iVar3 != 0) {
    if ((int)((uint)*(byte *)(iVar3 + 4) << 0x1d) < 0) {
      uVar6 = uVar6 | 0x10;
    }
    if ((int)((uint)*(byte *)(iVar3 + 4) << 0x1b) < 0) {
      uVar6 = uVar6 | 0x20;
    }
    if ((*(byte *)(iVar3 + 6) & 1) != 0) {
      uVar6 = uVar6 | 0x40;
    }
  }
  uVar4 = DAT_00004558[1] | uVar6;
  DAT_00004558[1] = (byte)uVar4;
  if (*(char *)(iVar2 + 0x1f) == '\0') {
    *(undefined1 *)(iVar2 + 0x1f) = 5;
    uVar5 = (uint)*pbVar1;
    if (uVar5 == 0xff) {
      *pbVar1 = (byte)uVar4;
      uVar7 = 1;
      pbVar1[1] = 0;
    }
    else {
      if (uVar5 != uVar4) {
        pbVar1[2] = (byte)(uVar5 ^ uVar4);
        if (((int)(uVar4 << 0x1c) < 0) && (((uVar5 ^ uVar4) & 3) != 0)) {
          pbVar1[2] = 0;
        }
      }
      pbVar1[1] = 0;
    }
    if (pbVar1[2] != 0) {
      fw_memcpy_bytes_ret(local_30,DAT_00004560,0x1c);
      uVar4 = 0;
      do {
        uVar5 = event_flag_edge_detect(local_30[uVar4],uVar6);
        uVar4 = uVar4 + 1 & 0xff;
        uVar7 = uVar7 | uVar5;
      } while (uVar4 < 7);
    }
  }
  else {
    pbVar1[2] = 0;
  }
  return uVar7 == 0;
}



/* ======================================================================
 * 000044a8  thunk_bab_rx_ba_session_ctl
 * ====================================================================== */

void thunk_bab_rx_ba_session_ctl(void)

{
  bab_rx_ba_session_ctl();
  return;
}



/* ======================================================================
 * 000044b0  ie_ssid_matches_list
 * ====================================================================== */

int ie_ssid_matches_list(undefined4 param_1,undefined4 param_2,int *param_3,uint param_4)

{
  int iVar1;
  int *piVar2;
  uint uVar3;
  int iVar4;
  
  iVar4 = 0;
  iVar1 = ie_find_in_frame(param_1,param_2,0,0);
  if (iVar1 != 0) {
    piVar2 = param_3;
    for (uVar3 = 0; iVar4 = 0, uVar3 < param_4; uVar3 = uVar3 + 1) {
      if (*piVar2 == 0) {
        return 1;
      }
      iVar4 = fw_mem_equal(iVar1 + 2,*(undefined1 *)(iVar1 + 1),piVar2 + 1);
      if (iVar4 != 0) {
        return iVar4;
      }
      piVar2 = piVar2 + 9;
    }
    uVar3 = 0;
    while ((uVar3 < param_4 &&
           (iVar4 = fw_mem_equal(param_3 + 1,*param_3,s_DIRECT__00004564,7), iVar4 == 0))) {
      param_3 = param_3 + 9;
      uVar3 = uVar3 + 1;
    }
  }
  return iVar4;
}



/* ======================================================================
 * 00004510  event_flag_edge_detect
 * ====================================================================== */

undefined4 event_flag_edge_detect(uint param_1,uint param_2)

{
  byte *pbVar1;
  byte bVar2;
  byte bVar3;
  byte *pbVar4;
  undefined4 uVar5;
  
  pbVar4 = DAT_00004558;
  uVar5 = 0;
  if ((DAT_00004558[2] & param_1) != 0) {
    bVar2 = *DAT_00004558;
    pbVar1 = DAT_00004558 + 2;
    bVar3 = (byte)param_1;
    if ((bVar2 & param_1) == 0) {
      if ((param_2 & param_1) == 0) {
        return 0;
      }
      *DAT_00004558 = bVar3 | bVar2;
    }
    else {
      if ((param_2 & param_1) != 0) {
        return 0;
      }
      *DAT_00004558 = bVar2 & ~bVar3;
    }
    uVar5 = 1;
    pbVar4[2] = *pbVar1 & ~bVar3;
  }
  return uVar5;
}



/* ======================================================================
 * 0000456c  lmc_tx_assign_default_rate
 * ====================================================================== */

void lmc_tx_assign_default_rate(xr_tx_ctx *param_1)

{
  byte bVar1;
  int iVar2;
  
  param_1->field_0x62 = 0xf;
  param_1->field_0xab = 0xff;
  param_1->field_0x60 = *(undefined1 *)(DAT_00004620 + (uint)param_1->bQueueId);
  *(undefined4 *)&param_1->field_0x64 = 0;
  *(uint *)&param_1->field_0x54 = param_1->dwHdr80211;
  txq_set_frame_lifetime(&param_1->field_0x54,0);
  bVar1 = param_1->field_0xc;
  if (bVar1 == 0xff) {
    if (1 < (byte)param_1->field_0xbd) {
      fw_assert(s_lmc_tx_c_00004624,0x19e,1000);
    }
    iVar2 = (uint)(byte)param_1->field_0xbd * 0x3b0 + DAT_00004630;
    if (*(char *)(iVar2 + 0x26) == '\0') {
      bVar1 = *(byte *)(iVar2 + 300);
    }
    else {
      bVar1 = *(byte *)(iVar2 + 0x12d);
    }
    if ((bVar1 == 0xff) || (*(byte *)(iVar2 + 0x27) < 3)) {
      bVar1 = *(byte *)(iVar2 + 0x24);
    }
  }
  if ((int)((uint)bVar1 << 0x18) < 0) {
    bVar1 = bVar1 & 0x7f;
  }
  else {
    *(uint *)&param_1->field_0x58 = *(uint *)&param_1->field_0x58 | 8;
  }
  param_1->field_0xc = bVar1;
  param_1->field_0x63 = bVar1;
  *(uint *)&param_1->field_0x80 = *(uint *)&param_1->field_0x80 | 2;
  tx_classify_hdr_len(param_1);
  txq_list_insert(param_1,param_1->bQueueId,2);
  evt_flags_set(DAT_00004634,0x200000);
  return;
}



/* ======================================================================
 * 00004638  phy_maybe_notify
 * ====================================================================== */

void phy_maybe_notify(void)

{
  if (*(char *)(DAT_00004648 + 1) != '\x02') {
    measure_arm_dwell_timer();
  }
  return;
}



/* ======================================================================
 * 000046a4  ps_decide_doze_or_awake
 * ====================================================================== */

longlong ps_decide_doze_or_awake(int param_1)

{
  char cVar1;
  byte bVar2;
  int iVar3;
  int iVar4;
  byte *pbVar5;
  byte bVar6;
  uint uStack_20;
  
  iVar3 = DAT_00004aa4;
  iVar4 = param_1 * 0x104 + DAT_00004aa4;
  pbVar5 = (byte *)(iVar4 + 0x40);
  cVar1 = *(char *)(iVar4 + 0xfc);
  *(undefined1 *)(iVar4 + 0xfc) = 0;
  if (*(char *)(iVar4 + 0x13e) == '\x01') {
    bVar6 = 0;
    *(undefined1 *)(iVar4 + 0x41) = 0;
  }
  else if (((*(char *)(iVar4 + 0x13e) == '\x02') ||
           ((*(byte *)(DAT_00004aa8 + 0x1c) & 0xf) >> 2 == 1)) || (2 < *(byte *)(iVar3 + 0x2a))) {
    bVar6 = 1;
  }
  else {
    bVar2 = *(byte *)(iVar4 + 0x124);
    *(byte *)(iVar4 + 0xfc) = bVar2 >> 7;
    bVar6 = bVar2 & 1;
    if ((*(char *)(iVar3 + 0x2a) == '\x01') && ((bVar2 & 1) == 0)) {
      *(undefined1 *)(iVar4 + 0xfc) = 1;
    }
  }
  ps_compute_intervals(param_1);
  if (*(char *)(iVar4 + 0xfc) != '\0') {
    if (*pbVar5 == 0) {
      if ((((*(byte *)(iVar4 + 0x58) & 0x12) != 0) || (cVar1 == '\0')) ||
         (-1 < (int)((uint)*(ushort *)(iVar4 + 0x44) << 0x1c))) goto LAB_0000473e;
    }
    else if ((*(ushort *)(iVar4 + 0x44) & 0x48) == 0) goto LAB_0000473e;
    bVar6 = 0;
  }
LAB_0000473e:
  if ((*(byte *)(iVar4 + 0x58) & 1) != 0) {
    *(undefined1 *)(iVar3 + 0x28) = 1;
  }
  *(undefined1 *)(iVar4 + 0x58) = 0;
  if (*pbVar5 == bVar6) {
    if (((bVar6 == 1) && (*(char *)(iVar4 + 0xfc) == '\0')) && (*(int *)(iVar4 + 0x128) != 0)) {
      *(undefined4 *)(iVar4 + 0x54) = *(undefined4 *)(iVar4 + 300);
    }
    if (*(char *)(iVar3 + 0x28) == '\0') {
      return (ulonglong)uStack_20 << 0x20;
    }
    if (*(char *)(DAT_00004aac + 0x11) == '\0') {
      ps_send_pm_complete_ind(pbVar5);
    }
    else {
      *(undefined1 *)(DAT_00004aac + 0x12) = 1;
    }
  }
  else {
    if (*(char *)(iVar4 + 0xfc) == '\0' && bVar6 == 0) {
      *(ushort *)(iVar4 + 0x44) = *(ushort *)(iVar4 + 0x44) & 0xfffe;
      *(undefined2 *)(iVar4 + 0x5a) = 0;
      timer_cancel(iVar4 + 0x98);
    }
    ps_set_mode_and_notify_ap(param_1,bVar6);
  }
  return CONCAT44(uStack_20,1);
}



/* ======================================================================
 * 000047ae  ps_reevaluate_all
 * ====================================================================== */

void ps_reevaluate_all(void)

{
  bool bVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  
  iVar2 = DAT_00004aa4;
  if (*(byte *)(DAT_00004aa4 + 0x2a) < 4) {
    if (*(byte *)(DAT_00004aa4 + 0x2a) == 3) {
      bVar1 = true;
      uVar3 = 0;
      do {
        iVar4 = uVar3 * 0x104 + iVar2;
        if ((*(char *)(iVar4 + 0x41) == '\0') || (*(char *)(iVar4 + 0x40) == '\x01')) {
          *(byte *)(iVar4 + 0x58) = *(byte *)(iVar4 + 0x58) & 0xfb;
        }
        else {
          bVar1 = false;
        }
        uVar3 = uVar3 + 1;
        *(undefined1 *)(iVar4 + 0xfc) = 0;
      } while (uVar3 < 2);
      if (bVar1) {
        *(undefined1 *)(iVar2 + 0x2a) = 4;
        ps_try_enter_sleep_all();
        return;
      }
    }
    if (-1 < (int)((uint)*(byte *)(DAT_00004aa8 + -0x95) << 0x1e)) {
      uVar3 = 0;
      do {
        if (*(short *)(uVar3 * 0x3b0 + DAT_00004ab0 + 0x42) == *(short *)(DAT_00004ab0 + 2)) break;
        uVar3 = uVar3 + 1 & 0xff;
      } while (uVar3 < 2);
      uVar5 = 0;
      do {
        if (1 < uVar3) {
          uVar3 = 0;
        }
        iVar4 = uVar3 * 0x104 + iVar2;
        if (*(char *)(iVar4 + 0x41) == '\0') {
          *(undefined1 *)(iVar4 + 0x58) = 0;
        }
        else if ((*(byte *)(iVar4 + 0x40) < 2) &&
                (iVar4 = ps_decide_doze_or_awake(uVar3), iVar4 != 0)) {
          return;
        }
        uVar3 = uVar3 + 1 & 0xff;
        uVar5 = uVar5 + 1;
        if (1 < uVar5) {
          return;
        }
      } while( true );
    }
  }
  return;
}



/* ======================================================================
 * 00004868  ps_release_radio_if_all_idle
 * ====================================================================== */

void ps_release_radio_if_all_idle(int param_1)

{
  int iVar1;
  
  iVar1 = param_1 * 0x104 + DAT_00004aa4;
  if (*(byte *)(DAT_00004aa4 + 0x2a) < 3) {
    if ((((*(char *)(iVar1 + 0x40) == '\x01') &&
         (*(short *)(param_1 * 0x3b0 + DAT_00004ab0 + 0x30) == 0)) &&
        (*(char *)(DAT_00004aa8 + -0x5b) == '\0')) && (*(short *)(iVar1 + 0x44) == 0)) {
      vif_release_radio_if_idle();
      return;
    }
  }
  else {
    ps_try_enter_sleep_all();
  }
  return;
}



/* ======================================================================
 * 000048b0  ps_wake_sequence
 * ====================================================================== */

void ps_wake_sequence(char *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  ushort *puVar2;
  int iVar3;
  undefined4 uVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  
  puVar2 = DAT_00004ab0;
  iVar1 = DAT_00004aa8;
  iVar5 = DAT_00004aa4;
  uVar7 = (uint)(byte)param_1[3];
  if (((*(char *)(DAT_00004aa8 + -0x5b) == '\0') && (*(byte *)(DAT_00004aa4 + 0x2a) < 4)) &&
     (*param_1 != '\0')) {
    if ((char)DAT_00004ab0[uVar7 * 0x1d8 + 0x33] != '\x03') {
      *(undefined1 *)(DAT_00004ab0 + uVar7 * 0x1d8 + 0x28) = 0x13;
      iVar3 = lmc_sched_request_radio(puVar2 + uVar7 * 0x1d8 + 0x22);
      if (iVar3 == 3) {
        return;
      }
    }
    if (*(char *)(iVar5 + 0x2b) != '\0') {
      uVar4 = fw_read_timer();
      *(undefined4 *)(param_1 + 8) = uVar4;
    }
    uVar4 = 2;
    if (((int)((uint)*DAT_00004ab0 << 0x1e) < 0) && ((byte)param_1[0xbf] < 4)) {
      uVar4 = 3;
    }
    phy_state_advance(uVar4);
    uVar4 = fw_read_timer();
    if (*(char *)(iVar5 + 0x2b) != '\0') {
      *(short *)(DAT_00004aa4 + 6) = (short)uVar4 - (short)*(undefined4 *)(param_1 + 8);
    }
    *(undefined4 *)(param_1 + 8) = uVar4;
    *(ushort *)(param_1 + 4) = *(ushort *)(param_1 + 4) | 0x80;
  }
  iVar5 = fw_read_timer();
  iVar5 = *(int *)(param_1 + 0xd0) - iVar5;
  if (iVar5 < 1) {
    if (DAT_00004ab4 <= iVar5 + DAT_00004ab4) goto LAB_0000495e;
    iVar3 = *(int *)(param_1 + 0xc) + iVar5;
  }
  else {
    iVar3 = iVar5 / 2 + *(int *)(param_1 + 0xc);
  }
  *(int *)(param_1 + 0xc) = iVar3;
LAB_0000495e:
  iVar6 = *(int *)(param_1 + 0xc4) + *(int *)(DAT_00004aa4 + 0x3c);
  iVar3 = iVar6 + *(int *)(param_1 + 200) * 2;
  if (0 < iVar5) {
    iVar3 = iVar3 + iVar5;
  }
  timer_start(param_1 + 0x44,iVar3,iVar6,*(int *)(DAT_00004aa4 + 0x3c),param_4);
  ps_resume_after_wake(uVar7,0);
  puVar2 = DAT_00004ab0;
  if ((((int)((uint)(byte)DAT_00004ab0[uVar7 * 0x1d8 + 0x19] << 0x1d) < 0) &&
      (*(byte *)(DAT_00004aa8 + 0x1d) < 2)) && (*(char *)(iVar1 + -0x59) == '\x01')) {
    *(undefined1 *)(iVar1 + -0x59) = 2;
    txp_prepare_all_pipes_idle();
    tx_arm_timeout(puVar2 + uVar7 * 0x1d8 + 0xc,DAT_00004ab8);
  }
  return;
}



/* ======================================================================
 * 000049b8  ps_schedule_next_tbtt_wake
 * ====================================================================== */

void ps_schedule_next_tbtt_wake(int param_1)

{
  undefined2 uVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  undefined4 uVar5;
  undefined4 uVar6;
  int iVar7;
  undefined8 uVar8;
  longlong lVar9;
  
  iVar7 = param_1 * 0x104 + DAT_00004aa4;
  iVar2 = fw_read_timer();
  iVar3 = DAT_00004abc;
  while (iVar3 <= iVar2 - *(int *)(DAT_00004aa4 + 0x34)) {
    *(int *)(DAT_00004aa4 + 0x34) = *(int *)(DAT_00004aa4 + 0x34) + iVar3;
    ps_keepalive_tick(param_1);
  }
  if (*(short *)(iVar7 + 0x10c) != 0) {
    iVar3 = param_1 * 0x3b0 + DAT_00004ab0;
    uVar8 = tsf_read(param_1);
    uVar6 = *(undefined4 *)(iVar3 + 0x118);
    lVar9 = __udivmoddi4((int)uVar8,(int)((ulonglong)uVar8 >> 0x20),uVar6,0);
    uVar5 = (undefined4)((ulonglong)lVar9 >> 0x20);
    if (*(char *)(iVar3 + 0x18) == '\x02') {
      lVar9 = u64_add_u32((int)lVar9,uVar5,1);
    }
    else {
      *(short *)(iVar7 + 0x134) = *(short *)(iVar7 + 0x134) + 1;
      uVar1 = __udivsi3(*(undefined4 *)(iVar3 + 0x3c0),*(ushort *)(iVar3 + 0x378));
      *(undefined2 *)(iVar7 + 0x136) = uVar1;
      if (*(char *)(DAT_00004ac0 + 8) == '\0') {
        lVar9 = lVar9 + (ulonglong)*(ushort *)(iVar3 + 0x378);
      }
      else {
        lVar9 = u64_add_u32((int)lVar9,uVar5,*(char *)(DAT_00004ac0 + 8));
      }
      uVar6 = *(undefined4 *)(iVar3 + 0x118);
    }
    iVar2 = u64_mul_acc_u32((int)lVar9,(int)((ulonglong)lVar9 >> 0x20),uVar6);
    iVar3 = DAT_00004aa4;
    iVar2 = ((iVar2 - (int)uVar8) + -0x14) -
            (*(uint *)(iVar7 + 0x108) + *(int *)(DAT_00004aa4 + 0x24));
    if ((*(char *)(iVar7 + 0x117) != '\0') && (*(uint *)(iVar7 + 0x108) < 4000)) {
      iVar2 = iVar2 + -2000;
    }
    iVar4 = fw_read_timer();
    *(int *)(iVar7 + 0x110) = iVar4 + iVar2;
    if (*(int *)(DAT_00004e98 + 0x30) == 0) {
      iVar4 = 10;
    }
    else {
      iVar4 = *DAT_00004e9c + 10;
    }
    *(undefined1 *)(iVar7 + 0x115) = 1;
    if (iVar4 < iVar2) {
      if (*(int *)(iVar7 + 0x4c) + DAT_00004ea0 < 0) {
        *(int *)(iVar7 + 0x4c) = -DAT_00004ea0;
      }
      iVar3 = *(int *)(iVar7 + 0x4c) + (iVar2 - iVar4);
      if (iVar3 < 1) {
        iVar3 = 0;
      }
      *(int *)(iVar7 + 0x100) = iVar3;
      timer_start(iVar7 + 0xd4);
      ps_release_radio_if_all_idle(param_1);
    }
    else {
      if (iVar2 < 0) {
        iVar2 = 0;
      }
      *(ushort *)(iVar7 + 0x44) = *(ushort *)(iVar7 + 0x44) | 0x80;
      timer_start(iVar7 + 0x84,
                  *(int *)(iVar7 + 0x104) + *(int *)(iVar3 + 0x3c) + *(int *)(iVar7 + 0x108) + iVar2
                 );
    }
  }
  return;
}



/* ======================================================================
 * 00004d0c  ps_beacon_rx_update_timing
 * ====================================================================== */

void ps_beacon_rx_update_timing(int *param_1)

{
  undefined1 uVar1;
  undefined4 uVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  int iVar7;
  char *pcVar8;
  uint uVar9;
  uint local_1c;
  undefined2 *local_18;
  
  uVar9 = (uint)*(byte *)((int)param_1 + 0x17);
  if (1 < uVar9) {
    return;
  }
  iVar7 = uVar9 * 0x104 + DAT_00004eac + -0x20;
  local_18 = DAT_00004ea4 + uVar9 * 0x1d8 + 0xc;
  if (((short)param_1[1] == *(short *)(iVar7 + 0x10c)) &&
     (*(char *)((int)param_1 + 0x16) == *(char *)(iVar7 + 0xff))) {
LAB_00004d92:
    if (*(char *)(iVar7 + 0x117) == '\0') goto LAB_00004d98;
  }
  else {
    *(short *)(iVar7 + 0x10c) = (short)param_1[1];
    uVar1 = *(undefined1 *)((int)param_1 + 0x16);
    *(undefined1 *)(iVar7 + 0xff) = uVar1;
    airtime_compute(&local_1c,0,uVar1,*DAT_00004ea4,*(short *)(iVar7 + 0x10c) + 4,1);
    *(uint *)(iVar7 + 0x104) = (local_1c & 0xffff) + DAT_00004ec0;
    iVar3 = DAT_00004eb0;
    *(undefined1 *)(iVar7 + 0x117) = 0;
    if (*(char *)(iVar3 + 6) == '\x01') {
      *(undefined1 *)(iVar3 + 6) = 2;
      uVar2 = fw_read_timer();
      *(undefined4 *)(DAT_00004eac + 0x14) = uVar2;
      goto LAB_00004d92;
    }
LAB_00004d98:
    if (*(char *)(iVar7 + 0x51) == '\0') {
      u64_add_u32(0,*(undefined4 *)(*param_1 + 0x1c),*(undefined4 *)(*param_1 + 0x18));
      uVar5 = *(uint *)(local_18 + 0x80);
      uVar6 = uVar5;
      __udivmoddi4();
      local_1c = uVar6;
      iVar3 = get_link_word_78(*(undefined1 *)((int)param_1 + 7),*(undefined1 *)((int)param_1 + 6));
      if (uVar5 >> 1 < (local_1c - iVar3) + 0x20) {
        *(undefined1 *)(iVar7 + 0x117) = 1;
      }
    }
  }
  *(undefined1 *)(iVar7 + 0x51) = 0;
  *(undefined4 *)(iVar7 + 0x108) = 0;
  uVar2 = fw_read_timer();
  *(undefined4 *)(iVar7 + 0x130) = uVar2;
  iVar4 = *(int *)(DAT_00004ec4 + 0x3c);
  iVar3 = 0x400;
  if ((0x400 < iVar4) || (iVar3 = -0x400, iVar4 < -0x400)) {
    iVar4 = iVar3;
  }
  pcVar8 = (char *)(DAT_00004eac + -0x20);
  iVar3 = *(int *)(DAT_00004eac + 4);
  if (iVar4 <= iVar3) {
    if (iVar3 <= iVar4) goto LAB_00004e14;
    iVar4 = (iVar3 + iVar4) / 2;
  }
  *(int *)(DAT_00004eac + 4) = iVar4;
LAB_00004e14:
  timer_cancel(iVar7 + 0x84);
  *(ushort *)(iVar7 + 0x44) = *(ushort *)(iVar7 + 0x44) & 0xff5f;
  if (*(char *)(DAT_00004eac + 0xb) != '\0') {
    *pcVar8 = *pcVar8 + '\x01';
    dbg_accumulate_wake_stats();
  }
  ps_resume_after_wake(uVar9,1);
  return;
}



/* ======================================================================
 * 00004e42  join_set_bssid_and_kick
 * ====================================================================== */

void join_set_bssid_and_kick(undefined4 *param_1)

{
  int iVar1;
  
  iVar1 = DAT_00004eac;
  *(undefined4 *)(DAT_00004eac + 0x18) = *param_1;
  *(byte *)(iVar1 + 0x38) = *(byte *)(iVar1 + 0x38) | 2;
  *(byte *)(DAT_00004ebc + 0x1c) = *(byte *)(DAT_00004ebc + 0x1c) | 2;
  ps_reevaluate_all();
  return;
}



/* ======================================================================
 * 00004e66  join_clear_state
 * ====================================================================== */

void join_clear_state(int param_1)

{
  int iVar1;
  int iVar2;
  
  iVar1 = DAT_00004eac;
  iVar2 = DAT_00004eac + -0x20;
  *(undefined4 *)(DAT_00004ec4 + 0x3c) = 0;
  iVar2 = param_1 * 0x104 + iVar2;
  *(undefined4 *)(iVar1 + 4) = 0;
  *(undefined1 *)(iVar2 + 0x13e) = 0;
  if (*(char *)(iVar2 + 0x59) != '\0') {
    *(byte *)(iVar2 + 0x58) = *(byte *)(iVar2 + 0x58) | 8;
    ps_reevaluate_all();
  }
  return;
}



/* ======================================================================
 * 00004f78  tx_pending_ctrl_frame_dispatch
 * ====================================================================== */

void tx_pending_ctrl_frame_dispatch(void)

{
  byte bVar1;
  uint uVar2;
  uint uVar3;
  uint uVar4;
  undefined4 in_r3;
  uint uVar5;
  
  uVar2 = 0;
  bVar1 = *(byte *)(DAT_00005000 + 0xb);
  uVar3 = (uint)bVar1;
  while( true ) {
    if (uVar3 == 0) {
      return;
    }
    if (1 < uVar2) {
      return;
    }
    uVar4 = uVar2 << 2;
    uVar5 = 2 << (uVar4 & 0xff);
    if ((uVar3 & uVar5 & 0xff) != 0) {
      *(byte *)(DAT_00005000 + 0xb) = bVar1 & ~(byte)uVar5;
      tx_send_null_data(uVar2 & 0xff,4);
      return;
    }
    uVar5 = 1 << (uVar4 & 0xff);
    if ((uVar3 & uVar5 & 0xff) != 0) break;
    uVar4 = 4 << (uVar4 & 0xff);
    if ((uVar3 & uVar4 & 0xff) != 0) {
      *(byte *)(DAT_00005000 + 0xb) = bVar1 & ~(byte)uVar4;
      tx_send_qos_null(uVar2 & 0xff,*(undefined1 *)(uVar2 * 0x104 + DAT_00004ff8 + 0x52),
                       DAT_00004ff8,uVar4,in_r3);
      return;
    }
    uVar2 = uVar2 + 1;
  }
  *(byte *)(DAT_00005000 + 0xb) = bVar1 & ~(byte)uVar5;
  tx_send_ps_poll(uVar2 & 0xff);
  return;
}



/* ======================================================================
 * 00005028  vif_apply_join_config
 * ====================================================================== */

longlong vif_apply_join_config(uint param_1)

{
  undefined1 uVar1;
  undefined2 uVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  int iVar8;
  char *pcVar9;
  uint uVar10;
  undefined4 uStack_20;
  
  iVar8 = param_1 * 0x3b0 + DAT_0000542c;
  pcVar9 = (char *)(iVar8 + 0x18);
  if (param_1 < 2) {
    uVar1 = link_slot_alloc();
    *(undefined1 *)(iVar8 + 0x12a) = uVar1;
  }
  iVar4 = DAT_0000542c;
  uVar3 = 0;
  uVar5 = param_1;
  uVar10 = param_1;
  do {
    iVar6 = uVar3 * 0x3b0 + DAT_0000542c;
    if (*(char *)(iVar6 + 0x19) == '\0') goto LAB_0000509a;
    uVar7 = *(uint *)(iVar6 + 0x1c);
    if ((uVar7 & 1) == 0) {
      if ((int)(uVar7 << 0x1c) < 0) {
        uVar7 = *(uint *)(iVar8 + 0x1c);
        if ((uVar7 & 0x1f) >> 2 == 0) goto LAB_00005078;
        goto LAB_000050cc;
      }
      if ((uVar7 & 0x14) != 0) {
        if (-1 < *(int *)(iVar8 + 0x1c) << 0x1c) {
          if (*(int *)(iVar8 + 0x1c) << 0x19 < 0) goto LAB_0000509a;
          goto LAB_00005096;
        }
        goto LAB_000050cc;
      }
    }
    else {
      uVar7 = *(uint *)(iVar8 + 0x1c);
LAB_00005078:
      if ((uVar7 & 1) == 0) {
        if (-1 < (int)(uVar7 << 0x19)) {
          uVar10 = uVar3 & 0xff;
        }
      }
      else {
LAB_00005096:
        uVar5 = uVar3 & 0xff;
      }
    }
LAB_0000509a:
    uVar3 = uVar3 + 1;
  } while (uVar3 < 2);
  syn_start_register_channel_use(pcVar9);
  iVar6 = DAT_00005430;
  if (*pcVar9 != '\0') {
    if (((*(uint *)(iVar8 + 0x1c) & 0x14) != 0) &&
       (*(int *)(iVar8 + 0x118) != *(int *)(uVar5 * 0x3b0 + iVar4 + 0x118))) {
LAB_000050cc:
      return (ulonglong)uStack_20 << 0x20;
    }
    *(char *)(DAT_00005430 + 0x19) = (char)uVar5;
    *(char *)(iVar6 + 0x1a) = (char)uVar10;
  }
  iVar6 = param_1 * 0x98 + DAT_00005434;
  *(undefined1 *)(iVar6 + 0x471) = *(undefined1 *)(iVar8 + 0x1b);
  *(char *)(iVar6 + 0x472) = *pcVar9;
  *(undefined4 *)(iVar6 + 0x478) = *(undefined4 *)(iVar8 + 0x28);
  *(undefined4 *)(iVar6 + 0x488) = 0;
  *(undefined4 *)(iVar6 + 0x48c) = 0;
  *(undefined2 *)(iVar6 + 0x482) = *(undefined2 *)(iVar8 + 0x3c);
  *(undefined2 *)(iVar6 + 0x484) = *(undefined2 *)(iVar8 + 0x3e);
  *(undefined2 *)(iVar6 + 0x486) = *(undefined2 *)(iVar8 + 0x40);
  vif_get_bssid(iVar8 + 0x34,*(byte *)(iVar8 + 0x1b) & 1);
  iVar4 = DAT_00005438;
  *(byte *)(iVar6 + 0x473) = *(byte *)(iVar8 + 0x1b) & 1;
  *(undefined2 *)(iVar6 + 0x47c) = *(undefined2 *)(iVar8 + 0x34);
  *(undefined2 *)(iVar6 + 0x47e) = *(undefined2 *)(iVar8 + 0x36);
  *(undefined2 *)(iVar6 + 0x480) = *(undefined2 *)(iVar8 + 0x38);
  uVar2 = phy_build_rate_cfg(*(undefined1 *)(iVar8 + 0x22),
                             *(uint *)(iVar4 + 0x30) | *(uint *)(iVar8 + 0x28),
                             *(undefined1 *)(iVar8 + 0x23),*(undefined2 *)(iVar8 + 0x42));
  *(undefined2 *)(iVar8 + 0x20) = uVar2;
  pas_pick_lowest_rate_from_mask(pcVar9);
  *(undefined1 *)(iVar8 + 0x27) = 3;
  *(uint *)(DAT_00005434 + 8) = *(uint *)(iVar8 + 0x118);
  if (*(int *)(iVar8 + 0x1c) << 0x1c < 0) {
    tsf_write(0,0);
  }
  if (((*(uint *)(iVar8 + 0x1c) & 0x14) != 0) && (param_1 != uVar5)) {
    *(uint *)(iVar6 + 0x488) = *(uint *)(iVar8 + 0x118) >> 1;
    *(undefined4 *)(iVar6 + 0x48c) = 0;
  }
  if (uVar10 != uVar5) {
    iVar4 = uVar10 * 0x98 + DAT_00005434;
    if ((*(char *)(iVar4 + 0x472) == '\x01') || (*(char *)(iVar4 + 0x472) == '\x05')) {
      *(byte *)(iVar4 + 0x471) = *(byte *)(iVar4 + 0x471) | 8;
      tsf_nudge_away_from_tbtt(uVar10);
    }
  }
  if (param_1 < 2) {
    *(undefined2 *)(iVar8 + 0x146) = *(undefined2 *)(iVar8 + 0x34);
    *(undefined2 *)(iVar8 + 0x148) = *(undefined2 *)(iVar8 + 0x36);
    *(undefined2 *)(iVar8 + 0x14a) = *(undefined2 *)(iVar8 + 0x38);
    *(undefined2 *)(iVar8 + 0x140) = *(undefined2 *)(iVar8 + 0x3c);
    *(undefined2 *)(iVar8 + 0x142) = *(undefined2 *)(iVar8 + 0x3e);
    *(undefined2 *)(iVar8 + 0x144) = *(undefined2 *)(iVar8 + 0x40);
    *(undefined2 *)(iVar8 + 0x14c) = *(undefined2 *)(iVar8 + 0x3c);
    *(undefined2 *)(iVar8 + 0x14e) = *(undefined2 *)(iVar8 + 0x3e);
    *(undefined2 *)(iVar8 + 0x150) = *(undefined2 *)(iVar8 + 0x40);
    if (*(int *)(iVar8 + 0x1c) << 0x1d < 0) {
      uVar2 = 0x200;
    }
    else if (*(int *)(iVar8 + 0x1c) << 0x1c < 0) {
      uVar2 = 0;
    }
    else {
      uVar2 = 0x100;
    }
    *(undefined2 *)(iVar8 + 0x13c) = uVar2;
  }
  iVar4 = DAT_0000543c;
  uVar5 = 0;
  *(undefined1 *)(iVar8 + 0x19) = 1;
  iVar8 = DAT_0000542c;
  *(undefined1 *)(iVar4 + 0xc) = 0;
  do {
    iVar6 = uVar5 * 0x3b0 + iVar8;
    if (*(char *)(iVar6 + 0x19) != '\0') {
      *(byte *)(iVar4 + 0xc) = (byte)(1 << *(sbyte *)(iVar6 + 0x22)) | *(byte *)(iVar4 + 0xc);
    }
    uVar5 = uVar5 + 1;
  } while (uVar5 < 3);
  return CONCAT44(uStack_20,1);
}



/* ======================================================================
 * 00005222  rx_beacon_join_match
 * ====================================================================== */

undefined4
rx_beacon_join_match(int *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  int *piVar5;
  
  iVar4 = *param_1;
  piVar5 = (int *)(DAT_00005438 + 0x40);
  if (-1 < *piVar5 << 0x1e) {
    iVar1 = ie_find_in_frame(iVar4,param_2,0,0,param_4);
    if (*(byte *)((int)param_1 + 0x17) < 2) {
      iVar3 = (uint)*(byte *)((int)param_1 + 0x17) * 0x3b0 + DAT_0000542c;
      if (((iVar1 != 0) &&
          (((iVar2 = fw_mem_equal(iVar1 + 2,*(undefined1 *)(iVar1 + 1),iVar3 + 0xf0,
                                  *(undefined4 *)(iVar3 + 0xec)), iVar2 != 0 ||
            (*(char *)(iVar1 + 2) == '\0')) || (*(char *)(iVar1 + 1) == '\0')))) &&
         (((*(short *)(iVar3 + 0x3c) == *(short *)(iVar4 + 0x10) &&
           (*(short *)(iVar3 + 0x3e) == *(short *)(iVar4 + 0x12))) &&
          (*(short *)(iVar3 + 0x40) == *(short *)(iVar4 + 0x14))))) {
        *piVar5 = 2;
        timer_cancel(DAT_0000543c + -0x40);
        *DAT_00005440 = *DAT_00005440 & 0xfffffeff;
        *(uint *)(iVar3 + 0x1c) = *(uint *)(iVar3 + 0x1c) | 0x100;
        mac_set_state3();
        if (*(char *)(iVar3 + 0x18) != '\x02') {
          if (*(int *)(iVar3 + 0x1c) << 0x13 < 0) {
            ind_080F_join_complete_a(*(undefined1 *)(iVar3 + 0x1a),0);
          }
          else {
            join_send_confirm_with_tsf(0);
          }
        }
        return 1;
      }
    }
  }
  return 0;
}



/* ======================================================================
 * 000052d4  vif_bss_event_timer_update
 * ====================================================================== */

void vif_bss_event_timer_update(int param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined4 local_20;
  undefined4 local_1c;
  int local_18;
  
  iVar2 = param_1 * 0x3b0 + DAT_0000542c;
  *(undefined1 *)(iVar2 + 0x3c4) = 0;
  if (*(char *)(iVar2 + 0x18) != '\x03') {
    iVar3 = param_1 * 0x3b0 + DAT_0000542c + 0xc4;
    local_18 = param_1;
    if (*(int *)(iVar2 + 0x3c0) == 0) {
      timer_cancel(iVar3);
      *(uint *)(iVar2 + 0x1c) = *(uint *)(iVar2 + 0x1c) & 0xfffffdff;
    }
    else {
      iVar1 = *(int *)(iVar2 + 0x118);
      if ((iVar1 != 0) && (*(int *)(iVar2 + 0x1c) << 0x17 < 0)) {
        timer_start(iVar3,*(int *)(iVar2 + 0x3c0) * iVar1);
      }
      if ((int)(*(uint *)(iVar2 + 0x1c) << 0x16) < 0) {
        *(uint *)(iVar2 + 0x1c) = *(uint *)(iVar2 + 0x1c) & 0xfffffdff;
        local_1c = 0;
        local_20 = 2;
        ind_0805_event_a(local_18,&local_20);
        timer_start(iVar3,*(int *)(iVar2 + 0x118) * *(int *)(iVar2 + 0x3c0));
        return;
      }
    }
  }
  return;
}



/* ======================================================================
 * 00005366  ap_send_probe_response
 * ====================================================================== */

undefined4 ap_send_probe_response(int *param_1)

{
  byte bVar1;
  ushort uVar2;
  bool bVar3;
  undefined1 uVar4;
  uint len;
  byte *buf;
  byte *pbVar5;
  int iVar6;
  byte *pbVar7;
  int iVar8;
  void *dst;
  void *src;
  int iVar9;
  uint uVar10;
  void *dst_00;
  int iVar11;
  bool bVar12;
  uint local_34;
  undefined4 local_30;
  
  bVar3 = false;
  if (*(byte *)((int)param_1 + 0x17) < 2) {
    iVar11 = *param_1;
    len = *(ushort *)(param_1 + 1) - 0x18;
    buf = (byte *)(iVar11 + 0x18);
    pbVar5 = ie_find(buf,len,0,0);
    if (pbVar5 != (byte *)0x0) {
      local_30 = 0;
      local_34 = 0;
LAB_0000539c:
      iVar9 = local_34 * 0x3b0 + DAT_0000542c;
      bVar12 = *(short *)(iVar9 + 0x42) == *(short *)(DAT_0000542c + 2);
      do {
        if ((!bVar12) ||
           (((iVar6 = *(int *)(iVar9 + 0x1c), -1 < iVar6 << 0x1c ||
             (*(int *)(DAT_00005434 + 0xc) == 0)) && (-1 < iVar6 << 0x1d)))) goto LAB_000055ac;
      } while ((((*(ushort *)(iVar11 + 0x10) != DAT_00005444) ||
                (*(ushort *)(iVar11 + 0x12) != DAT_00005444)) ||
               (*(ushort *)(iVar11 + 0x14) != DAT_00005444)) &&
              (((bVar12 = (uint)*(ushort *)(iVar11 + 0x10) == (uint)*(ushort *)(iVar9 + 0x3c),
                !bVar12 || (bVar12 = *(short *)(iVar11 + 0x12) == *(short *)(iVar9 + 0x3e), !bVar12)
                ) || (bVar12 = *(short *)(iVar11 + 0x14) == *(short *)(iVar9 + 0x40), !bVar12))));
      if (-1 < iVar6 << 0x1e) goto LAB_00005422;
      iVar6 = ie_peer_supports_ofdm(buf,len);
      bVar12 = iVar6 == 0;
LAB_000053fc:
      do {
        if (bVar12) goto LAB_000055ac;
        if ((((*(uint *)(iVar9 + 0x1c) & 7) >> 1 == 3) && (!bVar3)) &&
           (iVar6 = ie_find_p2p_vendor(buf,len & 0xffff), iVar6 != 0)) {
          bVar3 = true;
        }
LAB_00005422:
        if (*(int *)(iVar9 + 0x1c) << 6 < 0) {
          bVar1 = pbVar5[1];
          bVar12 = true;
          if ((bVar1 == 0) || ((bVar1 == 1 && (bVar12 = true, pbVar5[2] == 0)))) goto LAB_000053fc;
          iVar6 = fw_mem_equal(pbVar5 + 2,bVar1,s_DIRECT__000055c0,7);
          if (iVar6 != 0) {
            iVar6 = fw_mem_equal(pbVar5 + 2,pbVar5[1],iVar9 + 0xf0,*(undefined4 *)(iVar9 + 0xec));
            bVar12 = true;
            if (iVar6 == 0) goto LAB_000053fc;
          }
        }
        if ((pbVar5[1] == 0) ||
           (iVar6 = fw_mem_equal(pbVar5 + 2,pbVar5[1],iVar9 + 0xf0,*(undefined4 *)(iVar9 + 0xec)),
           iVar6 != 0)) goto LAB_000054aa;
        iVar6 = fw_mem_equal(pbVar5 + 2,pbVar5[1],s_DIRECT__000055c0,7);
        bVar12 = iVar6 == 0;
        if (!bVar12) goto LAB_000054aa;
      } while( true );
    }
  }
  return 0;
LAB_000054aa:
  iVar6 = local_34 * 0x40 + DAT_000055c8;
  if ((*(int *)(iVar9 + 0x3a4) << 0x1c < 0) ||
     (pbVar7 = ie_find(buf,len,10,0), pbVar7 != (byte *)0x0)) {
    local_30 = 1;
  }
  else if ((*(byte *)(DAT_000055cc + 4) < 2) && (iVar8 = tx_ctx_alloc_init(6,0,1), iVar8 != 0)) {
    if (((*(uint *)(iVar9 + 0x1c) & 7) >> 1 == 3) && (bVar3)) {
      beacon_update_noa_ie(local_34);
    }
    dst_00 = *(void **)(iVar8 + 0x1c);
    uVar2 = *(ushort *)(iVar6 + 0x2a);
    *(ushort *)(iVar8 + 0x5c) = uVar2;
    *(undefined1 *)(iVar8 + 0xc) = *(undefined1 *)(iVar6 + 0x29);
    fw_memcpy(dst_00,*(void **)(iVar6 + 0x2c),(uint)uVar2);
    *(undefined2 *)((int)dst_00 + 4) = *(undefined2 *)(iVar11 + 10);
    *(undefined2 *)((int)dst_00 + 6) = *(undefined2 *)(iVar11 + 0xc);
    *(undefined2 *)((int)dst_00 + 8) = *(undefined2 *)(iVar11 + 0xe);
    *(undefined2 *)((int)dst_00 + 10) = *(undefined2 *)(iVar9 + 0x34);
    *(undefined2 *)((int)dst_00 + 0xc) = *(undefined2 *)(iVar9 + 0x36);
    *(undefined2 *)((int)dst_00 + 0xe) = *(undefined2 *)(iVar9 + 0x38);
    if (*(int *)(iVar9 + 0x1c) << 0x1d < 0) {
      *(undefined2 *)((int)dst_00 + 0x10) = *(undefined2 *)(iVar9 + 0x34);
      *(undefined2 *)((int)dst_00 + 0x12) = *(undefined2 *)(iVar9 + 0x36);
      *(undefined2 *)((int)dst_00 + 0x14) = *(undefined2 *)(iVar9 + 0x38);
    }
    uVar4 = link_lookup_by_mac(local_34,iVar11 + 10);
    *(undefined1 *)(iVar8 + 0xbf) = uVar4;
    *(char *)(iVar8 + 0xbd) = (char)local_34;
    if (!bVar3) {
      uVar2 = *(ushort *)(iVar8 + 0x5c);
      dst = (void *)ie_find_p2p_vendor((int)dst_00 + 0x24,uVar2 - 0x24 & 0xffff);
      if (dst != (void *)0x0) {
        uVar10 = *(byte *)((int)dst + 1) + 2 & 0xff;
        src = (void *)((int)dst + uVar10);
        fw_memcpy(dst,src,(uint)uVar2 - ((int)src - (int)dst_00) & 0xffff);
        *(short *)(iVar8 + 0x5c) = *(short *)(iVar8 + 0x5c) - (short)uVar10;
      }
    }
    lmc_tx_assign_default_rate(iVar8);
  }
LAB_000055ac:
  local_34 = local_34 + 1 & 0xff;
  if (1 < local_34) {
    return local_30;
  }
  goto LAB_0000539c;
}



/* ======================================================================
 * 000055d0  beacon_update_noa_ie
 * ====================================================================== */

undefined4 beacon_update_noa_ie(int param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  uint uVar7;
  undefined4 local_2c;
  undefined4 local_28;
  uint local_24;
  uint local_20;
  uint local_1c;
  int local_18;
  
  local_28 = 0;
  local_2c = DAT_000056c8;
  uVar6 = 0;
  iVar5 = param_1 * 0x3b0 + DAT_000056cc;
  local_1c = (uint)*(byte *)(iVar5 + 0x1ad);
  local_24 = (uint)*(byte *)(iVar5 + 0x1ac);
  local_20 = (uint)*(byte *)(iVar5 + 0x182);
  local_18 = param_1 * 0x40 + DAT_000056d0;
  do {
    iVar1 = local_18 + (uint)*(byte *)((int)&local_2c + uVar6) * 8;
    if (*(short *)(iVar1 + 2) != 0) {
      iVar2 = p2p_find_noa_attr(iVar1);
      if (iVar2 == 0) {
        local_28 = 1;
      }
      else {
        if (local_1c == 0) {
          uVar7 = (uint)*(byte *)(iVar2 + 1) + (uint)*(byte *)(iVar2 + 2) * 0x100 + 3 & 0xffff;
          iVar3 = p2p_find_vendor_ie_in_template(iVar1);
          if (iVar3 != 0) {
            if ((int)(local_24 << 0x18) < 0) {
              if (*(ushort *)(iVar2 + 1) < 3) {
                uVar7 = 0;
              }
              else {
                uVar7 = 0xd;
              }
              iVar4 = *(ushort *)(iVar2 + 1) - uVar7;
              *(char *)(iVar2 + 1) = (char)iVar4;
              *(char *)(iVar2 + 2) = (char)((uint)iVar4 >> 8);
              iVar2 = iVar2 + 5;
            }
            template_remove_bytes(iVar1,iVar2,uVar7);
            *(char *)(iVar3 + 1) = *(char *)(iVar3 + 1) - (char)uVar7;
          }
        }
        else {
          *(char *)(iVar2 + 5) = (char)local_1c;
          fw_memcpy((void *)(iVar2 + 6),(void *)(iVar5 + 0x1b0),0xc);
          *(char *)(iVar2 + 3) = (char)local_20;
        }
        if ((*(char *)((int)&local_2c + uVar6) == '\x01') &&
           ((int)(*(uint *)(iVar5 + 0x1c) << 0xe) < 0)) {
          *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) | 0x400000;
        }
      }
    }
    uVar6 = uVar6 + 1 & 0xff;
  } while (uVar6 < 2);
  return local_28;
}



/* ======================================================================
 * 000056fc  tsf_timer_reload
 * ====================================================================== */

void tsf_timer_reload(void)

{
  undefined4 *puVar1;
  
  puVar1 = DAT_0000576c;
  DAT_0000576c[6] = DAT_00005768;
  puVar1[5] = 0xffffffff;
  puVar1[1] = 0xffffffff;
  puVar1[2] = (int)puVar1 << 9;
  puVar1[3] = *(undefined4 *)(DAT_00005770 + 8);
  *puVar1 = 1;
  return;
}



/* ======================================================================
 * 0000571a  tsf_accumulate_from_timer
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void tsf_accumulate_from_timer(void)

{
  int iVar1;
  
  iVar1 = DAT_00005770;
  *(undefined4 *)(DAT_00005770 + 8) = *(undefined4 *)(DAT_0000576c + 0xc);
  tsf_snapshot();
  *(int *)(iVar1 + 0x14) = *(int *)(iVar1 + 0x14) + _DAT_0ac00004;
  return;
}



/* ======================================================================
 * 00005736  tsf_resync
 * ====================================================================== */

void tsf_resync(void)

{
  int iVar1;
  
  iVar1 = tsf_estimate_drift();
  *(int *)(DAT_00005770 + 0x14) = iVar1 + *(int *)(DAT_00005770 + 0x14);
  hw_timer_init();
  tsf_timer_reload();
  return;
}



/* ======================================================================
 * 0000574e  hw_timer_init
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void hw_timer_init(void)

{
  _DAT_0ac00000 = 0xffffffff;
  _DAT_0ac00010 = 0x4f;
  _DAT_0ac00008 = 0x80;
  _DAT_0ac00024 = 0x4f;
  _DAT_0ac00020 = 0;
  return;
}



/* ======================================================================
 * 0000583e  tsf_snapshot
 * ====================================================================== */

void tsf_snapshot(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  iVar1 = DAT_00005968;
  if (*(char *)(DAT_00005968 + 0x1c) == '\0') {
    iVar2 = tsf_read_hw_counter();
  }
  else {
    iVar2 = tsf_read_hw_counter();
    iVar2 = iVar2 + *(int *)(iVar1 + 0x24);
  }
  *(int *)(iVar1 + 0x14) = iVar2;
  uVar3 = *(uint *)(DAT_00005954 + 0x80);
  if (*(int *)(DAT_00005954 + 0x6c) != *(int *)(DAT_00005954 + 0x6c)) {
    uVar3 = *(uint *)(DAT_00005954 + 0x80);
  }
  *(int *)(iVar1 + 4) = *(int *)(DAT_00005954 + 0x6c);
  *(uint *)(iVar1 + 8) = (uVar3 & 0x3fff) >> 1;
  return;
}



/* ======================================================================
 * 00005876  tsf_estimate_drift
 * ====================================================================== */

void tsf_estimate_drift(void)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  uint uVar4;
  undefined8 uVar5;
  
  iVar3 = DAT_00005968;
  uVar4 = *(uint *)(DAT_00005954 + 0x80);
  if (*(int *)(DAT_00005954 + 0x6c) != *(int *)(DAT_00005954 + 0x6c)) {
    uVar4 = *(uint *)(DAT_00005954 + 0x80);
  }
  iVar1 = *(int *)(DAT_00005954 + 0x6c) - *(int *)(DAT_00005968 + 4);
  if (iVar1 < 0) {
    iVar1 = iVar1 + 0x1000000;
  }
  if (*(char *)(DAT_00005968 + 0x1c) == '\0') {
    uVar5 = mul3(iVar1,*(undefined4 *)(DAT_00005968 + 0x14));
    uVar2 = (uint)uVar5 >> 4 | (int)((ulonglong)uVar5 >> 0x20) << 0x1c;
  }
  else {
    uVar5 = mul3(iVar1,*(undefined4 *)(DAT_00005968 + 0x14));
    iVar1 = __udivmoddi4((int)uVar5,(int)((ulonglong)uVar5 >> 0x20),0x300,0);
    uVar2 = iVar1 << 6;
  }
  iVar3 = ((uVar4 & 0x3fff) >> 1) - *(int *)(iVar3 + 8);
  if (iVar3 < 0) {
    iVar3 = -iVar3;
  }
  __udivsi3(uVar2 + iVar3 + 0x50,0xa0);
  return;
}



/* ======================================================================
 * 0000593a  tsf_read_hw_counter
 * ====================================================================== */

uint tsf_read_hw_counter(void)

{
  uint uVar1;
  
  if ((*(uint *)(DAT_00005954 + 0x84) & 1) == 0) {
    return *(uint *)(DAT_00005968 + 0xc);
  }
  uVar1 = *(uint *)(DAT_00005954 + 0x84) >> 0xd;
  *(uint *)(DAT_00005968 + 0xc) = uVar1;
  return uVar1;
}



/* ======================================================================
 * 000059a6  sched_arm_next_timer
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void sched_arm_next_timer(void)

{
  int iVar1;
  
  if (*DAT_000059d8 != 0) {
    iVar1 = *(int *)(*DAT_000059d8 + 8) - (_DAT_0ac00004 + *(int *)(DAT_000059dc + 0x14));
    if (iVar1 < 4) {
      evt_flags_set(DAT_000059e4,0x8000000);
      return;
    }
    _DAT_0ac0001c = 0xc1;
    _DAT_0ac00014 = iVar1;
  }
  return;
}



/* ======================================================================
 * 00005a3c  bab_rx_ba_session_ctl
 * ====================================================================== */

/* bab_rx_ba_session_ctl(if_id, arg) -- backend of WSM command 0x0014,
   host-driven RX block-ack (reorder buffer) control.  lmc_bab.c.
   
   arg layout:
     +0x00 u8  action     0 = delete, 3 = update window, other = delete+create
     +0x02 u8  tid
     +0x04 u8  mac[6]     (compared as three u16)
     +0x0A u16 seq        new window start, used when action == 3
   On return arg is overwritten with:
     +0x00 u32 status     0 ok, 2 = no matching/free slot
     +0x04 u32 free_slots = 4 - active_count
   
   Session table: **4 entries only**, 0x28 bytes each, at DAT_00005DD4+0x3A0.
   Per entry: +0x3A0 valid, +0x3A4..0x3A9 mac, +0x3AA tid, +0x3AB if_id,
   +0x3AC window start, +0x3AE window end, +0x3B0 window size,
   +0x3B4 reorder buffer.  Active count at DAT_00005DD4+0xD2.
   
   action 3 recomputes  end = (start + size - 1) & 0xFFF  -- a BAR / sequence
   number update.  Non-3 tears the session down (FUN_00002624, frees the
   reorder buffer) and, when action != 0, creates a new one via FUN_00005D38.
   
   *** Max 4 concurrent RX BA sessions. ***  mainline's cw1200_ampdu_action()
   returns -ENOTSUPP so it never drives this; the firmware instead sets
   sessions up itself from received ADDBA Action frames (bab_event_dispatch). */

void bab_rx_ba_session_ctl(uint param_1,char *param_2)

{
  int iVar1;
  ushort uVar2;
  uint uVar3;
  undefined4 uVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  
  iVar1 = DAT_00005dd4;
  uVar7 = 0;
  do {
    iVar6 = uVar7 * 0x28 + DAT_00005dd4;
    if ((((*(int *)(iVar6 + 0x3a0) != 0) && (*(byte *)(iVar6 + 0x3ab) == param_1)) &&
        (*(short *)(iVar6 + 0x3a4) == *(short *)(param_2 + 4))) &&
       (((*(short *)(iVar6 + 0x3a6) == *(short *)(param_2 + 6) &&
         (*(short *)(iVar6 + 0x3a8) == *(short *)(param_2 + 8))) &&
        (*(char *)(iVar6 + 0x3aa) == param_2[2])))) break;
    uVar7 = uVar7 + 1;
  } while (uVar7 < 4);
  if (*param_2 != '\x03') {
    if (uVar7 < 4) {
      pipe_clear_entry(uVar7);
      iVar6 = uVar7 * 0x28 + iVar1;
      timer_cancel(iVar6 + 0x3b4);
      *(undefined4 *)(iVar6 + 0x3a0) = 0;
      *(char *)(iVar1 + 0xd2) = *(char *)(iVar1 + 0xd2) + -1;
    }
    if (*param_2 != '\0') {
      uVar7 = bab_session_alloc_and_setup(param_1,param_2);
    }
    goto LAB_00005b12;
  }
  if (3 < uVar7) goto LAB_00005b12;
  iVar6 = uVar7 * 0x28 + DAT_00005dd4;
  uVar5 = (uint)*(ushort *)(param_2 + 10);
  uVar2 = *(ushort *)(iVar6 + 0x3ac);
  uVar3 = (uint)uVar2;
  if (DAT_00005dd8 < (int)(uVar3 + *(ushort *)(iVar6 + 0x3b0) + -1)) {
    if (uVar5 <= uVar3) {
LAB_00005ac2:
      if (*(ushort *)(iVar6 + 0x3ae) < uVar5) goto LAB_00005ac8;
    }
    uVar2 = *(ushort *)(param_2 + 10);
  }
  else if (uVar3 < uVar5) goto LAB_00005ac2;
LAB_00005ac8:
  *(ushort *)(iVar6 + 0x3ac) = uVar2;
  *(ushort *)(iVar6 + 0x3ae) = (uVar2 + *(ushort *)(iVar6 + 0x3b0)) - 1 & 0xfff;
LAB_00005b12:
  uVar4 = 0;
  if (3 < uVar7) {
    uVar4 = 2;
  }
  *(undefined4 *)param_2 = uVar4;
  *(uint *)(param_2 + 4) = 4 - (uint)*(byte *)(iVar1 + 0xd2);
  return;
}



/* ======================================================================
 * 00005b2a  bab_event_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00005b58) */
/* WARNING: Removing unreachable block (ram,0x00005b58) */
/* bab_event_dispatch() -- lmc_bab.c, block-ack session event pump.
   
   Drains a 16-entry ring of 0x2C-byte events (base DAT_00005DD4 + 0xE0,
   write index at +0xD3, read index at +0xD4, wrapped with & 0xF) and
   dispatches on the event type byte through the switch8 table at 0x00005B5C.
   
   The switch index is `type - 2`:
     2 -> bab_evt_type2 (0x000077B8)     3 -> bab_evt_type3 (0x00007AA4)
     4 -> inline at 0x00005BD4           5 -> bab_evt_type5 (0x000076D0)
     6 -> bab_evt_type6 (0x0000761C)     7 -> bab_evt_type7 (0x000079F4)
     default -> 0x00005C2A, which asserts lmc_bab.c:727 code 0x31
   
   The handlers build 802.11 Action frames -- bab_evt_type5 writes frame
   control 0x00D0 (type Management, subtype Action) -- i.e. this module runs
   ADDBA / DELBA / BAR signalling.
   
   No A-MPDU length decision is made here; that is TALA
   (tx_complete_tala_adapt, 0x0000D254).  What lmc_bab.c does own is the BA
   session state and window, which is a separate upper bound on aggregation
   and has NOT yet been checked against ampdu_num. */

void bab_event_dispatch(void)

{
  byte *pbVar1;
  uint uVar2;
  
  if ((-1 < (int)((uint)*(byte *)(DAT_00005dd4 + 0xd0) << 0x1e)) &&
     ((uint)*(byte *)(DAT_00005dd4 + 0xd3) != (uint)*(byte *)(DAT_00005dd4 + 0xd4))) {
    uVar2 = *(byte *)(DAT_00005dd4 + 0xd4) + 1 & 0xf;
    *(char *)(DAT_00005dd4 + 0xd4) = (char)uVar2;
    uVar2 = (uint)*(byte *)(uVar2 * 0x2c + DAT_00005dd4 + 0xe0);
                    /* WARNING: Could not recover jumptable at 0x00005b58. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    if (uVar2 - 2 < (uint)DAT_00005b5c) {
      pbVar1 = (byte *)(uVar2 + 0x5b5b);
    }
    else {
      pbVar1 = (byte *)(DAT_00005b5c + 0x5b5d);
    }
    (*(code *)((uint)*pbVar1 * 2 + 0x5b5d))();
    return;
  }
  return;
}



/* ======================================================================
 * 00005c5a  lmc_msg_alloc
 * ====================================================================== */

int lmc_msg_alloc(void)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  iVar1 = DAT_00005dd4;
  iVar3 = 0;
  irq_fiq_disable_save();
  uVar2 = *(byte *)(iVar1 + 0xd3) + 1 & 0xf;
  if (*(byte *)(iVar1 + 0xd4) != uVar2) {
    *(char *)(iVar1 + 0xd3) = (char)uVar2;
    iVar3 = uVar2 * 0x2c + iVar1 + 0xe0;
  }
  irq_fiq_restore();
  if (iVar3 == 0) {
    func_0xfff019c8(9,0);
  }
  return iVar3;
}



/* ======================================================================
 * 00005c92  rx_mgmt_queue_to_host
 * ====================================================================== */

undefined4 rx_mgmt_queue_to_host(void *param_1,uint param_2,byte param_3,int param_4,int param_5)

{
  uint *flags;
  int iVar1;
  char *pcVar2;
  undefined4 uVar3;
  char cVar4;
  
  cVar4 = '\0';
  if ((*(byte *)(DAT_00005dd4 + 0xd0) & 1) == 0) {
LAB_00005d34:
    uVar3 = 0;
  }
  else {
    if ((param_2 < 0x25) && (*(char *)((int)param_1 + 0x18) == '\x03')) {
      cVar4 = *(char *)((int)param_1 + 0x19);
      if (cVar4 == '\0') {
        cVar4 = *(char *)(param_5 * 0x3b0 + DAT_00005dfc + 0x18);
        if (((cVar4 == '\x04') || (cVar4 == '\x06')) &&
           (iVar1 = bab_should_buffer_for_link(param_5,param_1), iVar1 != 0)) {
          return 2;
        }
        cVar4 = '\x02';
        goto LAB_00005cde;
      }
      if (cVar4 == '\x01') {
        cVar4 = '\x03';
LAB_00005cec:
        pcVar2 = (char *)lmc_msg_alloc();
        if (pcVar2 != (char *)0x0) {
          fw_memcpy(pcVar2 + 4,param_1,param_2);
          if (param_4 == 4) {
            param_3 = param_3 | 0x80;
          }
          pcVar2[1] = param_3;
          *pcVar2 = cVar4;
          flags = DAT_00005de8;
          pcVar2[0x28] = (char)param_5;
          evt_flags_set(flags,0x400000);
        }
      }
      else if (cVar4 == '\x02') {
        cVar4 = '\x04';
        goto LAB_00005cec;
      }
    }
    else {
LAB_00005cde:
      if ((*(char *)((int)param_1 + 0x18) == '\0') || (*(char *)((int)param_1 + 0x18) == '\x05'))
      goto LAB_00005d34;
      if (cVar4 != '\0') goto LAB_00005cec;
    }
    uVar3 = 1;
  }
  return uVar3;
}



/* ======================================================================
 * 00005d38  bab_session_alloc_and_setup
 * ====================================================================== */

/* bab_session_alloc_and_setup(if_id, addba) -- install an RX block-ack session.
   
   Scans the 4-entry table at g_bab (stride 0x28) for a free slot
   (slot->in_use == 0 at +0x3A0) and fills it in:
   
     slot->bIfId   = if_id;                       /* +0x3AB */
     slot->in_use  = addba[0];                    /* +0x3A0 */
     slot->bTid    = addba[2];                    /* +0x3AA */
     slot->mac[6]  = addba[4..9];                 /* +0x3A4 */
     slot->start_seq = *(u16 *)&addba[10];        /* +0x3AC */
     slot->win_size  = MIN(addba[1], 0x10);       /* +0x3B0  <== CLAMP */
     slot->win_end   = (start_seq + win_size - 1) & 0xFFF;
     slot->timeout   = *(u16 *)&addba[12];        /* +0x3B2, units of 1024 us */
     if (timeout) timer_start(&slot->timer, timeout << 10);
     g_bab->count++;
   
   *** THE ADDBA BUFFER SIZE IS CLAMPED TO 16. ***
   `win_size = MIN(requested, 0x10)` — the firmware silently accepts a larger
   ADDBA request and reduces the window rather than refusing it.  This is the same
   16 that appears as the hardcoded A-MPDU subframe cap in
   txq_try_append_to_aggregate and as the 16 slot loop in bab_process_ba_bitmap, so
   **16 is enforced consistently at all three places**: session setup, aggregate
   build, and BlockAck completion.
   
   Consequence for the "raise the aggregate size" idea: patching only the build-side
   `cmp r0,#0x10` at 0x0A208 cannot work.  The completion path has 16 tracking slots
   and the session window is capped at 16 independently, so subframes 17+ would have
   nowhere to be recorded.  Three coordinated patches would be needed, not one.
   
   Only four sessions exist (bab_session_try_alloc), and mainline's
   cw1200_ampdu_action() returns -ENOTSUPP, so the firmware sets these up itself
   from received ADDBA frames -- the host never sees or controls the window. */

undefined8
bab_session_alloc_and_setup(int param_1,byte *param_2,undefined4 param_3,undefined4 param_4)

{
  byte bVar1;
  short sVar2;
  ushort uVar3;
  byte *pbVar4;
  int iVar5;
  uint uVar6;
  
  uVar6 = 0;
  do {
    iVar5 = uVar6 * 0x28 + DAT_00005dd4;
    if (*(uint *)(iVar5 + 0x3a0) == 0) {
      if (uVar6 < 4) {
        *(char *)(iVar5 + 0x3ab) = (char)param_1;
        *(uint *)(iVar5 + 0x3a0) = (uint)*param_2;
        bVar1 = param_2[2];
        *(byte *)(iVar5 + 0x3aa) = bVar1;
        *(undefined2 *)(iVar5 + 0x3a4) = *(undefined2 *)(param_2 + 4);
        *(undefined2 *)(iVar5 + 0x3a6) = *(undefined2 *)(param_2 + 6);
        *(undefined2 *)(iVar5 + 0x3a8) = *(undefined2 *)(param_2 + 8);
        uVar3 = 0x10;
        if (param_2[1] < 0x11) {
          uVar3 = (ushort)param_2[1];
        }
        sVar2 = *(short *)(param_2 + 10);
        *(short *)(iVar5 + 0x3ac) = sVar2;
        *(ushort *)(iVar5 + 0x3b0) = uVar3;
        *(ushort *)(iVar5 + 0x3ae) = (sVar2 + uVar3) - 1 & 0xfff;
        dup_cache_invalidate_exact(iVar5 + 0x3a4,(uint)bVar1 | param_1 << 8);
        pbVar4 = (byte *)(uint)*(ushort *)(iVar5 + 0x3ac);
        pipe_setup_entry(uVar6,param_1,*(undefined1 *)(iVar5 + 0x3aa),iVar5 + 0x3a4,pbVar4,
                         *(undefined2 *)(iVar5 + 0x3b0),param_4);
        uVar3 = *(ushort *)(param_2 + 0xc);
        *(ushort *)(iVar5 + 0x3b2) = uVar3;
        if (uVar3 != 0) {
          timer_start(iVar5 + 0x3b4,(uint)uVar3 << 10);
        }
        *(char *)(DAT_00005dd4 + 0xd2) = *(char *)(DAT_00005dd4 + 0xd2) + '\x01';
        param_2 = pbVar4;
      }
      break;
    }
    uVar6 = uVar6 + 1;
  } while (uVar6 < 4);
  return CONCAT44(param_2,uVar6);
}



/* ======================================================================
 * 00005e00  ps_cancel_and_refresh_tim
 * ====================================================================== */

void ps_cancel_and_refresh_tim(int param_1)

{
  undefined1 uVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  iVar2 = param_1 * 0x3b0 + DAT_000061fc;
  timer_cancel();
  uVar3 = *(uint *)(iVar2 + 0x1c);
  if ((int)(uVar3 * 0x400) < 0) {
    iVar4 = param_1 * 0x98 + DAT_00006200;
    *(uint *)(iVar2 + 0x1c) = uVar3 & 0xffdfffff;
    if ((int)((uVar3 & 0xffdfffff) * 8) < 0) {
      *(uint *)(iVar2 + 0x1c) = uVar3 & 0xefdfffff;
      *(byte *)(iVar4 + 0x492) = *(byte *)(iVar4 + 0x492) & 0xfe;
    }
    else {
      *(undefined1 *)(iVar4 + 0x493) = 0;
    }
  }
  uVar1 = *(undefined1 *)(iVar2 + 0x1ad);
  *(undefined1 *)(iVar2 + 0x1ad) = 0;
  beacon_update_noa_ie(param_1);
  *(undefined1 *)(iVar2 + 0x1ad) = uVar1;
  return;
}



/* ======================================================================
 * 00005e68  vif_ps_exit_cleanup
 * ====================================================================== */

void vif_ps_exit_cleanup(int param_1)

{
  undefined1 uVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  *(undefined1 *)(param_1 * 0x3b0 + DAT_000061fc + 0x1ad) = 0;
  iVar2 = param_1 * 0x3b0 + DAT_000061fc;
  timer_cancel();
  uVar3 = *(uint *)(iVar2 + 0x1c);
  if ((int)(uVar3 * 0x400) < 0) {
    iVar4 = param_1 * 0x98 + DAT_00006200;
    *(uint *)(iVar2 + 0x1c) = uVar3 & 0xffdfffff;
    if ((int)((uVar3 & 0xffdfffff) * 8) < 0) {
      *(uint *)(iVar2 + 0x1c) = uVar3 & 0xefdfffff;
      *(byte *)(iVar4 + 0x492) = *(byte *)(iVar4 + 0x492) & 0xfe;
    }
    else {
      *(undefined1 *)(iVar4 + 0x493) = 0;
    }
  }
  uVar1 = *(undefined1 *)(iVar2 + 0x1ad);
  *(undefined1 *)(iVar2 + 0x1ad) = 0;
  beacon_update_noa_ie(param_1);
  *(undefined1 *)(iVar2 + 0x1ad) = uVar1;
  return;
}



/* ======================================================================
 * 00005e7c  beacon_insert_p2p_noa_ie
 * ====================================================================== */

void beacon_insert_p2p_noa_ie(int param_1)

{
  ushort uVar1;
  int iVar2;
  int iVar3;
  uint n;
  int iVar4;
  void *dst;
  uint uVar5;
  undefined1 local_48;
  byte local_47;
  byte local_46;
  undefined1 local_45;
  undefined1 local_44;
  void *local_34;
  uint local_30;
  uint local_2c;
  uint local_28;
  int local_24;
  undefined4 local_20;
  int local_1c;
  int local_18;
  
  local_20 = DAT_00006204;
  uVar5 = 0;
  iVar2 = param_1 * 0x3b0 + DAT_000061fc;
  local_24 = iVar2 + 0x18;
  local_28 = (uint)*(byte *)(iVar2 + 0x1ad);
  local_30 = (uint)*(byte *)(iVar2 + 0x1ac);
  local_2c = (uint)*(byte *)(iVar2 + 0x182);
  local_1c = param_1 * 0x40 + DAT_00006208;
  local_18 = param_1;
  do {
    iVar2 = local_1c + (uint)*(byte *)((int)&local_20 + uVar5) * 8;
    if (((*(short *)(iVar2 + 2) != 0) && (iVar3 = p2p_find_noa_attr(iVar2), iVar3 == 0)) &&
       ((local_30 & 0xffffff80) != 0 || local_28 != 0)) {
      iVar3 = p2p_find_vendor_ie_in_template(iVar2);
      if (iVar3 == 0) break;
      local_48 = 0xc;
      local_47 = 2;
      local_45 = (undefined1)local_2c;
      local_44 = (undefined1)local_30;
      if (local_28 != 0) {
        local_47 = 0xf;
      }
      local_46 = 0;
      local_34 = (void *)((uint)*(byte *)(iVar3 + 1) + iVar3 + 2);
      n = (*(int *)(iVar2 + 4) + (uint)*(ushort *)(iVar2 + 2)) - (int)local_34 & 0xffff;
      iVar4 = tx_ctx_alloc_mgmt();
      if (iVar4 == 0) {
        return;
      }
      dst = *(void **)(iVar4 + 0x1c);
      fw_memcpy(dst,local_34,n);
      uVar1 = (ushort)local_47 + (ushort)local_46 * 0x100 + 3;
      fw_memcpy(local_34,&local_48,(uint)uVar1);
      fw_memcpy((void *)((int)local_34 + (uint)uVar1),dst,n);
      *(char *)(iVar3 + 1) = *(char *)(iVar3 + 1) + (char)uVar1;
      *(ushort *)(iVar2 + 2) = *(short *)(iVar2 + 2) + uVar1;
      tx_ctx_free(iVar4);
      if (*(char *)((int)&local_20 + uVar5) == '\x01') {
        *(uint *)(local_24 + 4) = *(uint *)(local_24 + 4) | 0x400000;
      }
    }
    uVar5 = uVar5 + 1 & 0xff;
  } while (uVar5 < 2);
  beacon_update_noa_ie(local_18);
  return;
}



/* ======================================================================
 * 00005fa8  beacon_arm_tbtt_timer
 * ====================================================================== */

void beacon_arm_tbtt_timer(int param_1,undefined4 param_2)

{
  int iVar1;
  int iVar2;
  bool bVar3;
  bool bVar4;
  undefined8 uVar5;
  
  iVar1 = param_1 * 0x3b0 + DAT_000061fc;
  timer_start(iVar1 + 0x198,param_2);
  uVar5 = tsf_read(param_1);
  uVar5 = u64_add_u32((int)uVar5,(int)((ulonglong)uVar5 >> 0x20),param_2);
  iVar2 = param_1 * 0x3b0 + DAT_000061fc;
  if (((int)((uint)*(byte *)(iVar1 + 0x1ac) * 0x1000000) < 0) &&
     (bVar3 = (*(uint *)(iVar1 + 0x1c) & 0x80000000) != 0, -1 < (int)(*(uint *)(iVar1 + 0x1c) << 1))
     ) {
    bVar4 = (int)uVar5 == 0;
    u64_cmp((int)uVar5,(int)((ulonglong)uVar5 >> 0x20),*(undefined4 *)(iVar2 + 0x178),
            *(undefined4 *)(iVar2 + 0x17c));
    if (bVar3 && !bVar4) {
      *(undefined8 *)(iVar2 + 0x178) = uVar5;
    }
  }
  else {
    *(undefined8 *)(iVar2 + 0x178) = uVar5;
  }
  iVar1 = param_1 * 0x98 + DAT_00006200;
  *(undefined4 *)(iVar1 + 0x474) = *(undefined4 *)(iVar2 + 0x178);
  *(undefined1 *)(iVar1 + 0x493) = 1;
  return;
}



/* ======================================================================
 * 00006032  beacon_tbtt_handler
 * ====================================================================== */

void beacon_tbtt_handler(int param_1)

{
  undefined4 uVar1;
  byte bVar2;
  undefined4 uVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  undefined8 uVar8;
  
  uVar7 = (uint)*(byte *)(param_1 + 2);
  uVar8 = tsf_read(uVar7);
  uVar3 = (undefined4)((ulonglong)uVar8 >> 0x20);
  uVar1 = (undefined4)uVar8;
  if ((*(uint *)(param_1 + 4) & 7) >> 1 == 3) {
    beacon_advance_tbtt_by_intervals(uVar7,uVar3,uVar1,uVar3);
  }
  uVar5 = DAT_00006200;
  uVar6 = *(uint *)(param_1 + 4);
  if ((int)(uVar6 * 8) < 0) {
    *(uint *)(param_1 + 4) = uVar6 & 0xefffffff;
    iVar4 = uVar7 * 0x98 + uVar5;
    bVar2 = *(byte *)(iVar4 + 0x492) & 0xfe;
    *(byte *)(iVar4 + 0x492) = bVar2;
    ps_wake_and_resume(uVar7,bVar2,uVar1,uVar3);
  }
  else if (*(char *)(param_1 + 0x195) != '\0') {
    *(uint *)(param_1 + 4) = uVar6 | 0x10000000;
    if ((uVar6 & 3) != 0) {
      iVar4 = uVar7 * 0x98 + uVar5;
      uVar5 = *(byte *)(iVar4 + 0x492) | 1;
      *(char *)(iVar4 + 0x492) = (char)uVar5;
    }
    beacon_tbtt_done(uVar7,uVar5,uVar1,uVar3);
    return;
  }
  return;
}



/* ======================================================================
 * 000060b6  beacon_catch_up_tbtt
 * ====================================================================== */

undefined4
beacon_catch_up_tbtt(int param_1,undefined4 param_2,uint param_3,int param_4,int *param_5)

{
  longlong lVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  int iVar5;
  undefined4 uVar6;
  int iVar7;
  uint uVar8;
  uint uVar9;
  int iVar10;
  char cVar11;
  char cVar12;
  longlong lVar13;
  undefined8 uVar14;
  undefined4 local_38;
  
  local_38 = 0;
  iVar7 = *param_5;
  iVar10 = param_5[1];
  uVar14 = *(undefined8 *)param_5;
  iVar2 = param_1 * 0x3b0 + DAT_000061fc;
  iVar3 = *(int *)(iVar2 + 0x1b4);
  uVar4 = *(undefined4 *)(iVar2 + 0x1b0);
  uVar9 = (uint)*(byte *)(iVar2 + 0x1ad);
  if (iVar3 == 0) {
    if (1 < uVar9) {
      fw_assert(s_lmc_p2p_c_00006210,DAT_0000620c,0x30);
    }
    lVar13 = u64_add_u32(iVar7,iVar10,uVar4);
    iVar5 = (int)((ulonglong)lVar13 >> 0x20);
    lVar1 = lVar13 - CONCAT44(param_4,param_3);
    cVar12 = SBORROW4(iVar5,param_4) != SBORROW4(iVar5 - param_4,(uint)((uint)lVar13 < param_3));
    cVar11 = '\0';
    u64_rsub_full((int)lVar1,(int)((ulonglong)lVar1 >> 0x20),0,0);
    if (cVar11 == cVar12) {
      uVar8 = 1;
      goto LAB_00006136;
    }
  }
  else {
    uVar8 = __udivsi3(param_3 - iVar7,iVar3);
    if (uVar8 == 0) goto LAB_0000618e;
LAB_00006136:
    if (uVar9 != 0xff) {
      if (uVar8 < uVar9) {
        uVar9 = uVar9 - uVar8 & 0xff;
      }
      else {
        uVar9 = 0;
      }
      *(char *)(iVar2 + 0x1ad) = (char)uVar9;
      if (uVar9 == 0) {
        ps_cancel_and_refresh_tim(param_1);
      }
    }
    uVar14 = u64_add_u32(iVar7,iVar10,iVar3 * uVar8);
    uVar6 = (undefined4)((ulonglong)uVar14 >> 0x20);
    lVar13 = u64_add_u32((int)uVar14,uVar6,uVar4);
    iVar5 = (int)((ulonglong)lVar13 >> 0x20);
    lVar1 = lVar13 - CONCAT44(param_4,param_3);
    cVar12 = SBORROW4(iVar5,param_4) != SBORROW4(iVar5 - param_4,(uint)((uint)lVar13 < param_3));
    cVar11 = '\0';
    u64_sub((int)lVar1,(int)((ulonglong)lVar1 >> 0x20),0,0);
    if (cVar11 != cVar12) {
      uVar14 = u64_add_u32((int)uVar14,uVar6,iVar3);
      goto LAB_0000618e;
    }
  }
  local_38 = 1;
LAB_0000618e:
  *(int *)(iVar2 + 0x1c8) = iVar7;
  *(int *)(iVar2 + 0x1cc) = iVar10;
  *(undefined8 *)param_5 = uVar14;
  return local_38;
}



/* ======================================================================
 * 000061a4  beacon_compute_next_dtim_tbtt
 * ====================================================================== */

int beacon_compute_next_dtim_tbtt(int param_1,undefined8 *param_2)

{
  byte bVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  undefined8 uVar7;
  undefined8 uVar8;
  undefined4 local_28;
  undefined4 local_24;
  
  iVar3 = param_1 * 0x3b0 + DAT_000061fc;
  iVar5 = *(int *)(iVar3 + 0x118);
  uVar7 = tsf_read(param_1);
  iVar4 = *(int *)(iVar3 + 0x118);
  iVar2 = iVar4;
  __udivmoddi4();
  uVar7 = s64_sub_s32((int)uVar7,(int)((ulonglong)uVar7 >> 0x20));
  uVar6 = (uint)*(byte *)(iVar3 + 0x164);
  bVar1 = *(byte *)(iVar3 + 0x110);
  if ((uint)(iVar4 - iVar2) < 0x401) {
    uVar7 = u64_add_u32((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),iVar5);
    if (uVar6 == 0) {
      uVar6 = bVar1 - 1 & 0xff;
    }
    else {
      uVar6 = uVar6 - 1 & 0xff;
    }
  }
  local_24 = (undefined4)((ulonglong)uVar7 >> 0x20);
  local_28 = (undefined4)uVar7;
  uVar8 = u64_add_u32(local_28,local_24,iVar5 * (uVar6 + 1));
  if (1 < *(byte *)(iVar3 + 0x1af)) {
    uVar8 = u64_add_u32((int)uVar8,(int)((ulonglong)uVar8 >> 0x20),
                        (uint)bVar1 * iVar5 * (*(byte *)(iVar3 + 0x1af) - 1));
  }
  uVar8 = u64_add_u32((int)uVar8,(int)((ulonglong)uVar8 >> 0x20),*(undefined4 *)(iVar3 + 0x1b8));
  iVar2 = tsf_read_low(param_1);
  *(int *)(iVar3 + 0x1b8) = (int)uVar8;
  *(char *)(iVar3 + 0x182) = *(char *)(iVar3 + 0x182) + '\x01';
  *(undefined8 *)(iVar3 + 0x1c0) = uVar8;
  *(undefined8 *)(iVar3 + 0x1c8) = uVar8;
  *param_2 = uVar7;
  return (int)uVar8 - iVar2;
}



/* ======================================================================
 * 00006294  ps_resync_beacon_state
 * ====================================================================== */

void ps_resync_beacon_state(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  int iVar2;
  uint uVar3;
  undefined4 uVar4;
  int iVar5;
  undefined8 uVar6;
  
  ps_cancel_and_refresh_tim();
  iVar5 = param_1 * 0x3b0 + DAT_00006698;
  if (*(char *)(iVar5 + 0x1ad) == '\0') {
    uVar1 = *(uint *)(iVar5 + 0x1c);
    uVar3 = uVar1 & 0xefffffff;
    *(uint *)(iVar5 + 0x1c) = uVar3;
    if (*(char *)(iVar5 + 0x1ac) == '\0') {
      if ((int)(uVar3 << 2) < 0) {
        *(uint *)(iVar5 + 0x1c) = uVar1 & 0xcfffffff;
        phy_state_advance(0);
      }
      *(undefined4 *)(iVar5 + 0x178) = 0;
      *(undefined4 *)(iVar5 + 0x17c) = 0;
      *(undefined1 *)(param_1 * 0x98 + DAT_0000669c + 0x493) = 0;
      timer_cancel(iVar5 + 0x184);
    }
    beacon_insert_p2p_noa_ie(param_1);
    return;
  }
  uVar1 = *(uint *)(iVar5 + 0x1c);
  if ((uVar1 & 7) >> 1 == 3) {
    beacon_resync_after_wake(param_1);
  }
  else if ((~uVar1 & 3) == 0) {
    *(uint *)(iVar5 + 0x1c) = uVar1 | 0x200000;
    uVar6 = tsf_read(param_1);
    uVar4 = (undefined4)((ulonglong)uVar6 >> 0x20);
    iVar2 = (int)uVar6;
    uVar6 = s64_add_s32(iVar2,uVar4,*(int *)(iVar5 + 0x1b8) - iVar2);
    *(undefined8 *)(iVar5 + 0x1c8) = uVar6;
    beacon_set_next_tbtt(param_1,(int)((ulonglong)uVar6 >> 0x20),iVar2,uVar4,uVar6,param_4);
    return;
  }
  return;
}



/* ======================================================================
 * 0000634a  ps_maybe_release_radio
 * ====================================================================== */

void ps_maybe_release_radio(int param_1)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  char cVar4;
  uint uVar5;
  ushort uVar6;
  
  iVar1 = DAT_0000669c;
  uVar3 = *(uint *)(param_1 + 4);
  uVar2 = (uint)*(byte *)(param_1 + 2);
  *(uint *)(param_1 + 4) = uVar3 | 0x40000000;
  if ((-1 < (int)(uVar3 * 0x400)) || ((int)(uVar3 << 3) < 0)) {
    *(undefined1 *)(uVar2 * 0x98 + iVar1 + 0x493) = 0;
  }
  if (-1 < (int)((uint)*(byte *)(param_1 + 0x194) << 0x18)) {
    return;
  }
  uVar3 = *(uint *)(param_1 + 4);
  if ((uVar3 & 7) >> 1 == 3) {
    uVar6 = 0;
    uVar5 = 0;
    do {
      if ((1 << uVar5 & (uint)*(ushort *)(param_1 + 0x144)) != 0) {
        uVar6 = uVar6 + 1 & 0xff;
      }
      uVar5 = uVar5 + 1 & 0xff;
    } while (uVar5 < 0x10);
    if (((*(ushort *)(DAT_000066a0 + 0x14) == uVar6) && (*(ushort *)(param_1 + 0x144) != 0)) &&
       (*(short *)(param_1 + 0x146) == 0)) {
      if (*(char *)(param_1 + 0x14b) == '\0') goto LAB_000063fa;
      *(uint *)(param_1 + 4) = uVar3 | 0x80000000;
    }
    cVar4 = '\0';
  }
  else {
    if ((~uVar3 & 3) != 0) {
      return;
    }
    cVar4 = *(char *)(uVar2 * 0x104 + DAT_000066a4 + 0x40);
  }
  if (cVar4 == '\0') {
    return;
  }
LAB_000063fa:
  *(uint *)(param_1 + 4) = *(uint *)(param_1 + 4) | 0x20000000;
  *(undefined4 *)(param_1 + 0x160) = 0;
  *(undefined4 *)(param_1 + 0x164) = 0;
  *(undefined1 *)(uVar2 * 0x98 + iVar1 + 0x493) = 0;
  if ((-1 < *DAT_000066a8 << 0x1a) && (*(short *)(param_1 + 0x18) == 0)) {
    vif_release_radio_if_idle();
  }
  return;
}



/* ======================================================================
 * 0000642e  ps_arm_listen_interval_timer
 * ====================================================================== */

void ps_arm_listen_interval_timer
               (int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  int iVar2;
  int iVar3;
  bool bVar4;
  bool bVar5;
  undefined8 uVar6;
  
  iVar2 = param_1 * 0x3b0 + DAT_00006698;
  *(uint *)(iVar2 + 0x1c) = *(uint *)(iVar2 + 0x1c) & 0xbfffffff;
  iVar3 = (*(byte *)(iVar2 + 0x1ac) & 0x7f) << 10;
  if (((*(byte *)(iVar2 + 0x1ac) & 0x7f) != 0) &&
     ((timer_start(iVar2 + 0x184,iVar3,param_3,0x3b0,param_4), -1 < *(int *)(iVar2 + 0x1c) << 0x1d
      || (*(short *)(iVar2 + 0x15c) != 0)))) {
    uVar6 = tsf_read(param_1);
    uVar6 = u64_add_u32((int)uVar6,(int)((ulonglong)uVar6 >> 0x20),iVar3);
    uVar1 = *(uint *)(iVar2 + 0x1c);
    iVar2 = param_1 * 0x3b0 + DAT_00006698;
    if (((int)(uVar1 * 0x400) < 0) && (bVar4 = (uVar1 & 0x20000000) != 0, -1 < (int)(uVar1 << 3))) {
      bVar5 = (int)uVar6 == 0;
      u64_cmp((int)uVar6,(int)((ulonglong)uVar6 >> 0x20),*(undefined4 *)(iVar2 + 0x178),
              *(undefined4 *)(iVar2 + 0x17c));
      if (bVar4 && !bVar5) {
        *(undefined8 *)(iVar2 + 0x178) = uVar6;
      }
    }
    else {
      *(undefined8 *)(iVar2 + 0x178) = uVar6;
    }
    iVar3 = param_1 * 0x98 + DAT_0000669c;
    *(undefined4 *)(iVar3 + 0x474) = *(undefined4 *)(iVar2 + 0x178);
    *(undefined1 *)(iVar3 + 0x493) = 1;
  }
  return;
}



/* ======================================================================
 * 000064d0  ps_resume_after_wake
 * ====================================================================== */

void ps_resume_after_wake(int param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  int iVar2;
  undefined4 local_18;
  undefined4 uStack_14;
  
  iVar2 = param_1 * 0x3b0 + DAT_00006698;
  uVar1 = *(uint *)(iVar2 + 0x1c);
  if (((int)(uVar1 << 0x1e) < 0) && (param_2 == 1)) {
    *(uint *)(iVar2 + 0x1c) = uVar1 & 0xdfffffff;
    local_18 = param_3;
    uStack_14 = param_4;
    if ((*(short *)(iVar2 + 0x30) != 0) || ((int)(uVar1 << 5) < 0)) {
      *(uint *)(iVar2 + 0x1c) = uVar1 & 0xdbffffff;
      evt_flags_set(DAT_000066ac,0x200000);
    }
    ps_arm_listen_interval_timer(param_1);
    if ((int)(*(uint *)(iVar2 + 0x1c) << 0xc) < 0) {
      *(uint *)(iVar2 + 0x1c) = *(uint *)(iVar2 + 0x1c) & 0xfff7ffff;
      local_18 = CONCAT22(local_18._2_2_,1);
      ind_080c_suspend_resume(*(undefined1 *)(iVar2 + 0x1a),&local_18);
    }
  }
  return;
}



/* ======================================================================
 * 0000652e  beacon_tbtt_done
 * ====================================================================== */

void beacon_tbtt_done(int param_1,undefined4 param_2,int param_3)

{
  int iVar1;
  int iVar2;
  undefined8 uVar3;
  
  iVar1 = param_1 * 0x3b0 + DAT_00006698;
  if ((-1 < (int)((uint)*(byte *)(iVar1 + 0x1ac) << 0x18)) || (*(int *)(iVar1 + 0x1c) << 1 < 0)) {
    *(undefined4 *)(iVar1 + 0x178) = 0;
    *(undefined4 *)(iVar1 + 0x17c) = 0;
    *(undefined1 *)(param_1 * 0x98 + DAT_0000669c + 0x493) = 0;
    if ((*(short *)(iVar1 + 0x30) == 0) &&
       ((-1 < *(int *)(iVar1 + 0x1c) << 0xb && (-1 < *DAT_000066a8 << 0x1a)))) {
      vif_release_radio_if_idle();
    }
  }
  iVar2 = *(int *)(iVar1 + 0x1c8);
  *(int *)(iVar1 + 0x1c0) = iVar2;
  *(undefined4 *)(iVar1 + 0x1c4) = *(undefined4 *)(iVar1 + 0x1cc);
  uVar3 = u64_add_u32(iVar2,*(undefined4 *)(iVar1 + 0x1cc),*(undefined4 *)(iVar1 + 0x1b4));
  *(undefined8 *)(iVar1 + 0x1c8) = uVar3;
  timer_start(iVar1 + 0x198,*(int *)(iVar1 + 0x1b0) - (param_3 - iVar2));
  return;
}



/* ======================================================================
 * 000065ae  ps_wake_and_resume
 * ====================================================================== */

void ps_wake_and_resume(int param_1,undefined4 param_2,int param_3,undefined4 param_4)

{
  char cVar1;
  int iVar2;
  undefined4 local_1c;
  undefined4 uStack_18;
  
  iVar2 = param_1 * 0x3b0 + DAT_00006698;
  local_1c = param_3;
  uStack_18 = param_4;
  if ((int)(*(uint *)(iVar2 + 0x1c) << 0xc) < 0) {
    *(uint *)(iVar2 + 0x1c) = *(uint *)(iVar2 + 0x1c) & 0xfff7ffff;
    local_1c = CONCAT22((short)((uint)param_3 >> 0x10),1);
    ind_080c_suspend_resume(*(undefined1 *)(iVar2 + 0x1a),&local_1c);
  }
  if (-1 < *(int *)(iVar2 + 0x1c) << 1) {
    vif_resume_tx_after_radio(param_1);
    if (*(short *)(iVar2 + 0x2e) != 0) {
      phy_state_advance(0);
    }
    *(uint *)(iVar2 + 0x1c) = *(uint *)(iVar2 + 0x1c) & 0xfbffffff;
    evt_flags_set(DAT_000066ac,0x200000);
  }
  cVar1 = *(char *)(iVar2 + 0x1ad);
  if (cVar1 != -1) {
    if (cVar1 != '\0') {
      *(char *)(iVar2 + 0x1ad) = cVar1 + -1;
      if (cVar1 != '\x01') goto LAB_00006622;
    }
    ps_cancel_and_refresh_tim(param_1);
    return;
  }
LAB_00006622:
  beacon_arm_tbtt_timer(param_1,*(int *)(iVar2 + 0x1c8) - param_3);
  return;
}



/* ======================================================================
 * 0000663c  beacon_advance_tbtt_by_intervals
 * ====================================================================== */

void beacon_advance_tbtt_by_intervals(int param_1,undefined4 param_2,uint param_3,int param_4)

{
  byte bVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  uint uVar5;
  int iVar6;
  undefined4 uVar7;
  int iVar8;
  char cVar9;
  bool bVar10;
  bool bVar11;
  undefined8 uVar12;
  
  iVar6 = param_1 * 0x3b0 + DAT_00006698;
  iVar8 = param_1 * 0x3b0 + DAT_00006698;
  uVar5 = *(uint *)(iVar6 + 0x1b8);
  param_4 = param_4 - (uint)(param_3 < uVar5);
  cVar9 = '\0';
  u64_cmp(param_3 - uVar5,param_4,0x80000000,0);
  if (cVar9 != '\0') {
    uVar7 = *(undefined4 *)(iVar6 + 0x1b4);
    uVar12 = __udivmoddi4(param_3 - uVar5,param_4,uVar7,0);
    uVar4 = (undefined4)((ulonglong)uVar12 >> 0x20);
    iVar2 = (int)uVar12;
    iVar3 = u64_mul_acc_u32(iVar2,uVar4,uVar7);
    *(char *)(iVar8 + 0x182) = *(char *)(iVar8 + 0x182) + '\x01';
    *(uint *)(iVar6 + 0x1b8) = iVar3 + uVar5;
    bVar1 = *(byte *)(iVar6 + 0x1ad);
    bVar11 = 0xfe < bVar1;
    if (bVar1 != 0xff) {
      bVar10 = iVar2 == 0;
      u64_cmp(bVar1,0,iVar2,uVar4);
      if (!bVar11 || bVar10) {
        *(undefined1 *)(iVar6 + 0x1ad) = 0;
      }
      else {
        *(byte *)(iVar6 + 0x1ad) = bVar1 - (char)uVar12;
      }
    }
    beacon_update_noa_ie(param_1);
  }
  return;
}



/* ======================================================================
 * 000066ec  beacon_set_next_tbtt
 * ====================================================================== */

void beacon_set_next_tbtt
               (int param_1,undefined4 param_2,uint param_3,int param_4,uint param_5,int param_6)

{
  int iVar1;
  undefined4 extraout_r1;
  int iVar2;
  int iVar3;
  char cVar4;
  char cVar5;
  
  iVar2 = param_1 * 0x3b0 + DAT_000068b0;
  iVar3 = param_1 * 0x3b0 + DAT_000068b0;
  *(uint *)(iVar2 + 0x178) = param_5;
  *(int *)(iVar2 + 0x17c) = param_6;
  cVar5 = SBORROW4(param_4,param_6) != SBORROW4(param_4 - param_6,(uint)(param_3 < param_5));
  cVar4 = '\0';
  u64_sub(param_3 - param_5,(param_4 - param_6) - (uint)(param_3 < param_5),0,0);
  if ((cVar4 == cVar5) &&
     (iVar1 = beacon_catch_up_tbtt(param_1,extraout_r1,param_3,param_4,&param_5), iVar1 != 0)) {
    beacon_tbtt_handler(iVar3 + 0x18);
    return;
  }
  timer_start(iVar3 + 0x198,param_5 - param_3);
  iVar3 = param_1 * 0x98 + DAT_000068b4;
  *(undefined4 *)(iVar3 + 0x474) = *(undefined4 *)(iVar2 + 0x178);
  *(undefined1 *)(iVar3 + 0x493) = 1;
  return;
}



/* ======================================================================
 * 00006778  beacon_resync_after_wake
 * ====================================================================== */

void beacon_resync_after_wake(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined4 uVar1;
  int iVar2;
  int iVar3;
  undefined4 local_18;
  undefined4 local_14;
  
  iVar3 = param_1 * 0x3b0 + DAT_000068b0;
  if ((int)(*(uint *)(iVar3 + 0x1c) << 0xe) < 0) {
    *(uint *)(iVar3 + 0x1c) = *(uint *)(iVar3 + 0x1c) | 0x200000;
    local_18 = param_3;
    local_14 = param_4;
    uVar1 = beacon_compute_next_dtim_tbtt(param_1,&local_18);
    timer_start(iVar3 + 0x198,uVar1);
    beacon_insert_p2p_noa_ie(param_1);
    *(undefined4 *)(iVar3 + 0x17c) = *(undefined4 *)(iVar3 + 0x1cc);
    iVar2 = DAT_000068b4;
    *(undefined4 *)(iVar3 + 0x178) = *(undefined4 *)(iVar3 + 0x1c8);
    iVar2 = param_1 * 0x98 + iVar2;
    *(undefined4 *)(iVar2 + 0x474) = *(undefined4 *)(iVar3 + 0x1c8);
    *(undefined1 *)(iVar2 + 0x493) = 1;
    beacon_count_missed_tbtts(param_1,iVar2 + 0x480,local_18,local_14);
  }
  return;
}



/* ======================================================================
 * 000067e4  beacon_count_missed_tbtts
 * ====================================================================== */

void beacon_count_missed_tbtts(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  char cVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  undefined4 uVar5;
  char cVar6;
  undefined8 uVar7;
  undefined8 uVar8;
  undefined4 local_38;
  
  iVar3 = param_1 * 0x3b0 + DAT_000068b0;
  if (*(char *)(iVar3 + 0x1ad) == '\x01') {
    iVar2 = *(int *)(iVar3 + 0x118);
    uVar4 = 1;
    cVar1 = '\0';
    while( true ) {
      uVar7 = u64_add_u32(param_3,param_4,iVar2 * uVar4);
      uVar5 = (undefined4)((ulonglong)uVar7 >> 0x20);
      cVar6 = '\0';
      uVar8 = u64_add_u32((int)uVar7,uVar5,0x400);
      u64_cmp((int)uVar8,(int)((ulonglong)uVar8 >> 0x20),*(undefined4 *)(iVar3 + 0x178),
              *(undefined4 *)(iVar3 + 0x17c));
      if (cVar6 != '\0') break;
      uVar4 = uVar4 + 1 & 0xff;
    }
    uVar8 = u64_add_u32(*(undefined4 *)(iVar3 + 0x178),*(undefined4 *)(iVar3 + 0x17c),
                        *(undefined4 *)(iVar3 + 0x1b0));
    u64_cmp((int)uVar8,(int)((ulonglong)uVar8 >> 0x20),(int)uVar7,uVar5);
    if ((cVar6 != '\0') && (uVar4 != 0)) {
      *(char *)(iVar3 + 0x165) = (char)uVar4 + -1;
      *(uint *)(iVar3 + 0x1c) = *(uint *)(iVar3 + 0x1c) | 0x8000000;
    }
    while( true ) {
      uVar5 = (undefined4)((ulonglong)uVar7 >> 0x20);
      local_38 = (undefined4)uVar7;
      cVar6 = (*(uint *)(iVar3 + 0x1c) & 0x10000000) != 0;
      if (-1 < (int)(*(uint *)(iVar3 + 0x1c) << 4)) break;
      uVar7 = u64_add_u32(*(undefined4 *)(iVar3 + 0x178),*(undefined4 *)(iVar3 + 0x17c),
                          *(undefined4 *)(iVar3 + 0x1b0));
      u64_cmp((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),local_38,uVar5);
      if (cVar6 == '\0') {
        *(char *)(iVar3 + 0x181) = cVar1;
        return;
      }
      cVar1 = cVar1 + '\x01';
      uVar7 = u64_add_u32(local_38,uVar5,iVar2);
    }
  }
  return;
}



/* ======================================================================
 * 000068b8  tx_rearm_or_expire
 * ====================================================================== */

void tx_rearm_or_expire(int param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  iVar1 = *(int *)(param_1 + 0x90);
  iVar3 = *(int *)(param_1 + 0x94);
  iVar2 = fw_read_timer();
  iVar2 = (iVar3 + iVar1) - iVar2;
  if (0 < iVar2 + -0x200) {
    tx_arm_timeout(param_1,iVar2);
    return;
  }
  *(undefined4 *)(param_1 + 0x90) = 0;
  return;
}



/* ======================================================================
 * 000068e6  vif_tbtt_post_process
 * ====================================================================== */

void vif_tbtt_post_process(uint param_1)

{
  int iVar1;
  ushort uVar2;
  int iVar3;
  
  iVar1 = DAT_00006acc;
  iVar3 = param_1 * 0x3b0 + DAT_00006acc;
  if (param_1 == 2) {
    *DAT_00006ad0 = 1;
  }
  if (*(short *)(iVar1 + 2) != *(short *)(iVar3 + 0x42)) {
    mac_program_channel_for_vifs();
  }
  if ((int)((uint)*(byte *)(iVar3 + 0x50) << 0x1a) < 0) {
    *(undefined1 *)(iVar3 + 0x50) = 0x30;
  }
  phy_state_advance(0);
  if (*(byte *)(iVar3 + 0x50) >> 4 == 1) {
    if (*(byte *)(iVar3 + 0x50) == 0x13) {
      ps_wake_sequence(param_1 * 0x104 + DAT_00006ad4 + 0x40);
    }
    else {
      beacon_tx_or_reschedule(iVar3 + 0x18);
    }
  }
  if ((((*(char *)(DAT_00006ad8 + 0x1d) == '\x01') &&
       (*(byte *)(*(int *)(DAT_00006ad8 + -0x50) + 0xd) == param_1)) &&
      (*(int *)(iVar3 + 0xa4) == 0)) && (*(int *)(iVar3 + 0xa8) != 0)) {
    tx_rearm_or_expire(iVar3 + 0x18);
  }
  *(ushort *)(iVar3 + 0x2e) = *(ushort *)(iVar3 + 0x2c);
  if (param_1 < 2) {
    uVar2 = *(ushort *)(iVar3 + 0x2c) & (~*(ushort *)(iVar3 + 0x15c) | *(ushort *)(iVar3 + 0x160));
    *(ushort *)(iVar3 + 0x2e) = uVar2;
    *(ushort *)(iVar3 + 0x2e) = uVar2 | *(ushort *)(iVar3 + 0x15e);
  }
  evt_flags_set(DAT_00006adc,0x200000);
  return;
}



/* ======================================================================
 * 0000699a  lmc_sched_radio_release
 * ====================================================================== */

/* lmc_sched_radio_release(ctx) -- lmc_sched.c, radio-ownership scheduler.
   
   Called from tx_complete_tala_adapt (0x0000D254) when the TX queue drains,
   and from four other sites.  Defers if the vif still has work
   (vif[0x30] != 0), otherwise releases the radio and hands it to the next
   waiting vif: scans up to 3 vifs for one whose vif[0x52] matches the
   released vif[0x42] and whose state byte vif[0x66] == 2, else pops the
   pending list at ctx[-0x54].
   
   Asserts lmc_sched.c:377 code 1000 if the released context is not the one
   currently recorded as owning the radio.
   
   This is inter-VIF / channel arbitration, not per-packet TX scheduling.
   With a single STA vif it is effectively a no-op, so it is not a
   throughput factor in the bench configuration. */

void lmc_sched_radio_release(int param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  undefined4 uVar4;
  int iVar5;
  
  iVar1 = DAT_00006ad8;
  iVar5 = (uint)*(byte *)(param_1 + 0xd) * 0x3b0 + DAT_00006acc;
  if (*(short *)(iVar5 + 0x30) != 0) {
    *(int *)(DAT_00006ad8 + -0x4c) = param_1;
    return;
  }
  *(undefined4 *)(DAT_00006ad8 + -0x4c) = 0;
  if (*(int *)(iVar1 + -0x58) != param_1) {
    fw_assert(s_lmc_sched_c_00006ae0,0x179,1000);
  }
  iVar3 = DAT_00006aec;
  *(undefined4 *)(iVar1 + -0x58) = 0;
  *(undefined1 *)(param_1 + 0x22) = 0;
  *(byte *)(iVar3 + 0x15) = *(byte *)(iVar3 + 0x15) & 0xfd;
  evt_flags_set(DAT_00006adc,0x200000);
  *(undefined2 *)(iVar5 + 0x2e) = 0;
  uVar2 = 0;
  while ((iVar3 = uVar2 * 0x3b0 + DAT_00006acc, *(short *)(iVar3 + 0x52) != *(short *)(iVar5 + 0x42)
         || (*(char *)(iVar3 + 0x66) != '\x02'))) {
    uVar2 = uVar2 + 1;
    if (2 < uVar2) {
      iVar5 = *(int *)(iVar1 + -0x54);
      if (iVar5 == 0) {
        return;
      }
      uVar4 = *(undefined4 *)(iVar5 + 4);
      *(int *)(iVar1 + -0x58) = iVar5;
      *(undefined4 *)(iVar1 + -0x54) = uVar4;
      *(undefined1 *)(iVar5 + 0x22) = 3;
      vif_tbtt_post_process(*(undefined1 *)(iVar5 + 0xd));
      return;
    }
  }
  *(int *)(iVar1 + -0x58) = iVar3 + 0x44;
  *(undefined1 *)(iVar3 + 0x66) = 3;
  return;
}



/* ======================================================================
 * 00006a3a  tx_timeout_recover
 * ====================================================================== */

void tx_timeout_recover(int param_1)

{
  int iVar1;
  
  iVar1 = DAT_00006ad8;
  if (*(int *)(DAT_00006ad8 + -0x50) != 0) {
    *(undefined4 *)(param_1 + 0x8c) = 0;
    if ((int)((uint)*(byte *)(param_1 + 0x1a) << 0x1d) < 0) {
      *(undefined1 *)(DAT_00006ad8 + 0x1d) = 1;
      if (*(short *)(param_1 + 0x2a) == *(short *)(DAT_00006acc + 2)) {
        if (*(int *)(param_1 + 0x90) != 0) {
          tx_rearm_or_expire();
          return;
        }
      }
      else {
        lmc_sched_request_radio(*(undefined4 *)(iVar1 + -0x50));
      }
    }
  }
  return;
}



/* ======================================================================
 * 00006a78  vif_release_radio_if_idle
 * ====================================================================== */

void vif_release_radio_if_idle(int param_1)

{
  int iVar1;
  
  iVar1 = param_1 * 0x3b0 + DAT_00006acc;
  if (((2 < *(byte *)(iVar1 + 0x66)) &&
      (lmc_sched_radio_release(iVar1 + 0x44), *(int *)(DAT_00006ad8 + -0x58) == 0)) &&
     (*(char *)(DAT_00006aec + 0x16) == '\x04')) {
    phy_enter_state3_checked();
  }
  return;
}



/* ======================================================================
 * 00006aac  ps_check_pending_wake
 * ====================================================================== */

void ps_check_pending_wake(void)

{
  if ((int)((uint)*(byte *)(DAT_00006ad8 + 0x1c) << 0x1d) < 0) {
    *(undefined1 *)(DAT_00006ad8 + -0x59) = 1;
  }
  else if ((*(byte *)(DAT_00006ad8 + 0x1c) & 1) != 0) {
    ps_evaluate_all_vifs();
    return;
  }
  return;
}



/* ======================================================================
 * 00006af0  beacon_tx_or_reschedule
 * ====================================================================== */

void beacon_tx_or_reschedule(int param_1)

{
  undefined1 uVar1;
  undefined4 *puVar2;
  int *piVar3;
  char cVar4;
  int iVar5;
  int iVar6;
  
  piVar3 = DAT_00006bc8;
  puVar2 = DAT_00006bc4;
  uVar1 = *(undefined1 *)(param_1 + 2);
  while (*piVar3 << 0x10 < 0) {
    *puVar2 = 0;
    task_143a6();
  }
  *(uint *)(param_1 + 4) = *(uint *)(param_1 + 4) | 0x100000;
  tsf_read(uVar1);
  iVar6 = *(int *)(DAT_00006bcc + 8);
  if (iVar6 == 0) {
    iVar5 = 0;
  }
  else {
    iVar5 = iVar6;
    __udivmoddi4();
  }
  if (iVar6 - iVar5 < 0xfa1) {
    if (*(char *)((int)DAT_00006bc4 + -0x79) == '\x01') {
      *(undefined2 *)(param_1 + 0x16) = 0;
    }
    if (*(char *)(param_1 + 0x4e) != '\x03') {
      *(undefined1 *)(param_1 + 0x38) = 0x10;
      iVar6 = lmc_sched_request_radio(param_1 + 0x2c);
      if (iVar6 == 3) {
        return;
      }
      phy_state_advance(0);
    }
    if ((int)(*(uint *)(param_1 + 4) << 9) < 0) {
      *(uint *)(param_1 + 4) = *(uint *)(param_1 + 4) & 0xffbfffff;
      syn_start_load_beacon_template(uVar1,0);
    }
    if (*(int *)(param_1 + 4) << 0x1d < 0) {
      cVar4 = *(char *)(param_1 + 0x14c);
      if (cVar4 == '\0') {
        cVar4 = *(char *)(param_1 + 0xf8);
      }
      *(char *)(param_1 + 0x14c) = cVar4 + -1;
      beacon_fill_tim(uVar1);
    }
    iVar6 = tsf_align_request(uVar1);
    if (iVar6 != 0) {
      beacon_rx_post_process(uVar1);
      return;
    }
    iVar6 = 0x400;
  }
  else {
    iVar6 = (iVar6 - iVar5) + -4000;
  }
  timer_start(param_1 + 0xc0,iVar6);
  return;
}



/* ======================================================================
 * 00006bd0  txpipe_clear_by_mac_tid
 * ====================================================================== */

void txpipe_clear_by_mac_tid(uint param_1,uint param_2,short *param_3)

{
  char *pcVar1;
  byte bVar2;
  
  bVar2 = 0;
  pcVar1 = DAT_00006c1c;
  while( true ) {
    if ((byte)DAT_00006c1c[-0xc] <= bVar2) {
      return;
    }
    if ((((*pcVar1 != '\0') && (*param_3 == *(short *)(pcVar1 + 0x18))) &&
        (param_3[1] == *(short *)(pcVar1 + 0x1a))) &&
       (((param_3[2] == *(short *)(pcVar1 + 0x1c) && ((byte)pcVar1[0x1e] == param_2)) &&
        ((byte)pcVar1[0x1f] == param_1)))) break;
    pcVar1 = pcVar1 + 0x38;
    bVar2 = bVar2 + 1;
  }
  *pcVar1 = '\0';
  return;
}



/* ======================================================================
 * 00006c20  wake_restore_context
 * ====================================================================== */

void wake_restore_context(void)

{
  undefined4 *puVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  int iVar5;
  int iVar6;
  
  *DAT_00006cdc = 0;
  puVar1 = DAT_00006ce4;
  iVar2 = DAT_00006ce0;
  iVar5 = *(int *)(DAT_00006ce0 + 0x3c);
  DAT_00006ce4[1] = *(undefined4 *)(DAT_00006ce0 + 0x38);
  iVar2 = *(int *)(iVar2 + 0x3c);
  puVar1[2] = iVar2;
  if (iVar2 != iVar5) {
    puVar1[1] = 0;
  }
  iVar2 = fw_read_timer();
  puVar1[3] = puVar1[1] - iVar2;
  *puVar1 = *(undefined4 *)(DAT_00006ce0 + 100);
  iVar5 = DAT_00006cf0;
  iVar2 = DAT_00006cec;
  if (*(char *)(DAT_00006ce8 + 0x1e) == '\0') {
    uVar3 = 0;
    do {
      iVar6 = uVar3 * 2;
      uVar3 = uVar3 + 1;
      *(undefined2 *)((int)puVar1 + iVar6 + 0x90) = *(undefined2 *)(iVar6 + iVar5 + iVar2);
    } while (uVar3 < 2);
    uVar3 = 0;
    do {
      uVar4 = uVar3 + 1;
      puVar1[uVar3 + 4] = *(undefined4 *)(uVar3 * 4 + iVar5 + 0x7000);
      uVar3 = uVar4;
    } while (uVar4 < 0x20);
    func_0x0000271a();
  }
  regs_restore_ctx();
  *(undefined4 *)(DAT_00006ce8 + -0x10) = 0;
  return;
}



/* ======================================================================
 * 00006c96  tsf_hw_start
 * ====================================================================== */

void tsf_hw_start(void)

{
  int iVar1;
  undefined4 *puVar2;
  undefined4 *puVar3;
  int iVar4;
  int iVar5;
  
  mac_hw_reset_regs();
  tsf_hw_init();
  iVar1 = DAT_00006ce0;
  *(undefined4 *)(DAT_00006ce0 + 0x34) = 1;
  iVar4 = fw_read_timer();
  puVar2 = DAT_00006ce4;
  iVar5 = DAT_00006ce4[2];
  if (0 < DAT_00006ce4[1] - (iVar4 + DAT_00006ce4[3])) {
    iVar5 = iVar5 + 1;
  }
  *(int *)(iVar1 + 0x38) = iVar4 + DAT_00006ce4[3];
  *(int *)(iVar1 + 0x3c) = iVar5;
  *(undefined4 *)(iVar1 + 0x34) = 3;
  iVar1 = DAT_00006cf4;
  puVar3 = DAT_00006ce8;
  *(undefined4 *)(DAT_00006cf4 + 8) = *DAT_00006ce8;
  *(undefined4 *)(iVar1 + 0xc) = puVar3[1];
  *(undefined4 *)(DAT_00006ce0 + 100) = *puVar2;
  *(undefined1 *)((int)puVar3 + 0x1e) = 1;
  return;
}



/* ======================================================================
 * 00006d30  wsm_h_14_impl
 * ====================================================================== */

void wsm_h_14_impl(undefined2 *param_1)

{
  undefined4 uVar1;
  
  thunk_bab_rx_ba_session_ctl(*(undefined1 *)(DAT_00006d54 + 10),param_1 + 2);
  *param_1 = 0xc;
  uVar1 = wsm_status_from_internal(*(undefined4 *)(param_1 + 2));
  *(undefined4 *)(param_1 + 2) = uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00006d7e  bab_set_session_state_by_mac
 * ====================================================================== */

void bab_set_session_state_by_mac(undefined4 param_1,int param_2)

{
  int iVar1;
  
  iVar1 = pipe_find_by_mac_upper();
  if (iVar1 != 0) {
    if ((int)((uint)*(byte *)(iVar1 + 6) << 0x1d) < 0) {
      if (param_2 == 0) {
        *(undefined1 *)(iVar1 + 6) = 3;
        evt_flags_set(DAT_000070b8,0x200000);
      }
    }
    else if (param_2 != 0) {
      *(undefined1 *)(iVar1 + 6) = 5;
      return;
    }
  }
  return;
}



/* ======================================================================
 * 00006dae  bab_mark_session_state5
 * ====================================================================== */

void bab_mark_session_state5(int param_1)

{
  int iVar1;
  
  if (((*(short *)(DAT_000070bc + 0x12) != 0) &&
      (iVar1 = pipe_find_by_mac_upper(*(int *)(param_1 + 0x54) + 4), iVar1 != 0)) &&
     (-1 < (int)((uint)*(byte *)(iVar1 + 6) << 0x1d))) {
    *(undefined1 *)(iVar1 + 6) = 5;
  }
  return;
}



/* ======================================================================
 * 00006dd0  ap_unmap_link
 * ====================================================================== */

undefined4 ap_unmap_link(uint param_1,short *param_2)

{
  char *pcVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  short *psVar5;
  int iVar6;
  int iVar7;
  uint uVar8;
  
  iVar3 = DAT_000070c4;
  iVar6 = param_1 * 0x3b0 + DAT_000070c0;
  uVar2 = 0;
  do {
    if (*(ushort *)(iVar3 + 0x14) <= uVar2) goto LAB_00006edc;
    iVar4 = uVar2 * 0xc + DAT_000070c0;
    iVar7 = iVar4 + DAT_000070c8;
    if (*(byte *)(iVar7 + 0x19) == param_1) {
      if (param_2 == (short *)0x0) {
        *(ushort *)(iVar6 + 0x2c) =
             *(ushort *)(iVar6 + 0x2c) & ~(ushort)(1 << *(sbyte *)(iVar7 + 0x18));
        *(undefined1 *)(iVar7 + 0x18) = 0;
      }
      else if (((*(short *)(iVar7 + 0x1e) == *param_2) &&
               (psVar5 = (short *)(iVar4 + DAT_000070c8 + 0x20), *psVar5 == param_2[1])) &&
              (psVar5[1] == param_2[2])) {
        *(ushort *)(iVar6 + 0x2c) =
             *(ushort *)(iVar6 + 0x2c) &
             ~(ushort)(1 << *(sbyte *)(uVar2 * 0xc + DAT_000070c0 + DAT_000070c8 + 0x18));
        *(undefined1 *)(iVar7 + 0x18) = 0;
        *(undefined1 *)(iVar7 + 0x1b) = 0;
        link_slot_free(*(undefined1 *)(iVar7 + 0x1a));
        uVar2 = 0;
        do {
          txpipe_clear_by_mac_tid(param_1,uVar2 & 0xff,param_2);
          uVar2 = uVar2 + 1;
        } while (uVar2 < 8);
        uVar2 = 0;
        do {
          iVar7 = uVar2 * 0x28 + DAT_000070cc;
          if (((*(int *)(iVar7 + 0x3a0) != 0) && (*(byte *)(iVar7 + 0x3ab) == param_1)) &&
             ((*(short *)(iVar7 + 0x3a4) == *param_2 &&
              ((*(short *)(iVar7 + 0x3a6) == param_2[1] && (*(short *)(iVar7 + 0x3a8) == param_2[2])
               ))))) {
            pipe_clear_entry(uVar2);
            timer_cancel(iVar7 + 0x3b4);
            *(undefined4 *)(iVar7 + 0x3a0) = 0;
            *(char *)(DAT_000070cc + 0xd2) = *(char *)(DAT_000070cc + 0xd2) + -1;
          }
          uVar2 = uVar2 + 1;
        } while (uVar2 < 4);
LAB_00006edc:
        uVar8 = 0;
        uVar2 = (uint)*(ushort *)(iVar3 + 0x14);
        while (uVar8 < uVar2) {
          pcVar1 = (char *)(uVar8 * 0xc + DAT_000070c0 + DAT_000070c8 + 0x18);
          if (*pcVar1 == '\0') {
            uVar2 = uVar2 - 1;
            if (uVar2 != 0) {
              memcpy_fast(pcVar1,uVar2 * 0xc + DAT_000070c0 + DAT_000070c8 + 0x18,0xc);
            }
          }
          else {
            uVar8 = uVar8 + 1;
          }
        }
        *(short *)(iVar3 + 0x14) = (short)uVar2;
        if ((int)(*(uint *)(iVar6 + 0x1c) << 0x1a) < 0) {
          iVar3 = 0;
          for (uVar8 = 0; uVar8 < (uVar2 & 0xffff); uVar8 = uVar8 + 1) {
            iVar7 = uVar8 * 0xc + DAT_000070c0 + DAT_000070c8;
            if ((*(byte *)(iVar7 + 0x19) == param_1) && (*(char *)(iVar7 + 0x18) != '\0')) {
              iVar3 = iVar3 + 1;
            }
          }
          if (iVar3 == 0) {
            *(uint *)(iVar6 + 0x1c) = *(uint *)(iVar6 + 0x1c) & 0xffffffdf;
            *(byte *)(iVar6 + 0x1b) = *(byte *)(iVar6 + 0x1b) | 4;
            iVar3 = param_1 * 0x98 + DAT_000070d0;
            *(byte *)(iVar3 + 0x471) = *(byte *)(iVar3 + 0x471) | 4;
            lmc_recompute_vif_roles();
            if (param_2 != (short *)0x0) {
              join_clear_state(param_1);
            }
          }
        }
        return 0;
      }
    }
    uVar2 = uVar2 + 1;
  } while( true );
}



/* ======================================================================
 * 00006f88  ap_map_link
 * ====================================================================== */

/* ap_map_link(struct wsm_map_link *arg) -- WSM 0x001C MAP_LINK backend.
   arg = { u8 mac[6]; u8 unmap; u8 link_id; } (matches cw1200 exactly).
   
     if (arg->unmap & 1) return ap_unmap_link(if_id, arg);
   
     if (link_count >= 0x10 || arg->link_id >= 0x0F) return 1;   /* FULL */
     if (an entry with the same (if_id, link_id) exists) return 2; /* DUP */
     ... install entry, then:
         vif[0x2C]  |=  1 << link_id;     /* mapped-link bitmap, u16 */
         vif[0x15C] &= ~(1 << link_id);
         if (vif[0x2E]) vif[0x2E] |= 1 << link_id;
         link_count++;
     return 0;
   
   *** FIRMWARE LIMITS: at most 16 mapped links, and link_id must be 0..14. ***
   mainline caps AP mode at CW1200_MAX_STA_IN_AP_MODE = 5; the vendor tree
   uses MAX_STA_IN_AP_MODE = 14.  The firmware is the vendor's number, so
   mainline is leaving 9 stations on the table.  Careful when raising it: the
   host's pseudo link ids (AFTER_DTIM, UAPSD) must also stay below 15, and the
   mapped bitmap at vif+0x2C is only 16 bits.
   
   tx_lmac_req_submit rejects TX with status 0x14 for any link_id not set in
   vif[0x2C], so an unmapped link fails cleanly rather than crashing.
   
   Per-link inactivity is seeded here from vif[0x3A0]/vif[0x3A1], which are
   written by MIB 0x1035 SET_INACTIVITY -- and only when BOTH are non-zero.
   Since the value is copied at map time, SET_INACTIVITY only affects links
   mapped after it is written.  Mainline never writes it, so firmware STA
   inactivity monitoring is off. */

undefined8 ap_map_link(undefined2 *param_1,undefined4 param_2)

{
  byte bVar1;
  byte bVar2;
  int iVar3;
  int iVar4;
  undefined1 uVar5;
  undefined4 uVar6;
  uint uVar7;
  int iVar8;
  undefined2 *puVar9;
  int iVar10;
  uint uVar11;
  int iVar12;
  undefined4 local_1c;
  
  iVar4 = DAT_000070c8;
  iVar3 = DAT_000070c4;
  iVar10 = DAT_000070c0;
  bVar1 = *(byte *)(DAT_000070c0 + 10);
  uVar11 = (uint)bVar1;
  if ((*(byte *)(param_1 + 3) & 1) != 0) {
    uVar6 = ap_unmap_link(uVar11,param_1);
    return CONCAT44(param_2,uVar6);
  }
  if ((*(ushort *)(DAT_000070c4 + 0x14) < 0x10) &&
     (bVar2 = *(byte *)((int)param_1 + 7), bVar2 < 0xf)) {
    local_1c = 0;
    for (uVar7 = 0; uVar7 < *(ushort *)(DAT_000070c4 + 0x14); uVar7 = uVar7 + 1 & 0xff) {
      iVar8 = uVar7 * 0xc + DAT_000070c0 + DAT_000070c8;
      if ((*(byte *)(iVar8 + 0x19) == uVar11) && (*(byte *)(iVar8 + 0x18) == bVar2)) {
        local_1c = 2;
        goto LAB_000070aa;
      }
    }
    iVar8 = uVar7 * 0xc + DAT_000070c0;
    iVar12 = iVar8 + DAT_000070c8;
    *(byte *)(iVar12 + 0x18) = bVar2;
    *(byte *)(iVar12 + 0x19) = bVar1;
    puVar9 = (undefined2 *)(iVar8 + iVar4 + 0x20);
    *(undefined2 *)(iVar12 + 0x1e) = *param_1;
    *puVar9 = param_1[1];
    puVar9[1] = param_1[2];
    iVar10 = uVar11 * 0x3b0 + iVar10;
    uVar5 = link_slot_alloc();
    *(undefined1 *)(iVar12 + 0x1a) = uVar5;
    if ((*(char *)(iVar10 + 0x3a1) != '\0') && (*(char *)(iVar10 + 0x3a0) != '\0')) {
      *(char *)(iVar12 + 0x1b) = *(char *)(iVar10 + 0x3a0) + *(char *)(iVar10 + 0x3a1);
      *(undefined1 *)(iVar12 + 0x1c) = 0;
      *(undefined1 *)(iVar12 + 0x1d) = 0;
    }
    *(ushort *)(iVar10 + 0x2c) =
         (ushort)(1 << *(sbyte *)((int)param_1 + 7)) | *(ushort *)(iVar10 + 0x2c);
    *(ushort *)(iVar10 + 0x15c) =
         *(ushort *)(iVar10 + 0x15c) & ~(ushort)(1 << *(sbyte *)((int)param_1 + 7));
    if (*(ushort *)(iVar10 + 0x2e) != 0) {
      *(ushort *)(iVar10 + 0x2e) =
           *(ushort *)(iVar10 + 0x2e) | (ushort)(1 << *(sbyte *)((int)param_1 + 7));
      evt_flags_set(DAT_000070b8,0x200000);
    }
    dup_cache_invalidate_by_mac(uVar11,param_1);
    *(short *)(iVar3 + 0x14) = *(short *)(iVar3 + 0x14) + 1;
    if ((*(uint *)(iVar10 + 0x1c) & 0x23) == 1) {
      *(uint *)(iVar10 + 0x1c) = *(uint *)(iVar10 + 0x1c) | 0x20;
      *(byte *)(iVar10 + 0x1b) = *(byte *)(iVar10 + 0x1b) & 0xfb;
      iVar10 = uVar11 * 0x98 + DAT_000070d0;
      *(byte *)(iVar10 + 0x471) = *(byte *)(iVar10 + 0x471) & 0xfb;
      ps_force_awake(uVar11);
      lmc_recompute_vif_roles();
    }
  }
  else {
    local_1c = 1;
  }
LAB_000070aa:
  return CONCAT44(param_2,local_1c);
}



/* ======================================================================
 * 000070d4  rx_probe_resp_matches_our_ssid
 * ====================================================================== */

undefined4 rx_probe_resp_matches_our_ssid(int *param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  char cVar4;
  undefined8 uVar5;
  undefined8 uVar6;
  
  iVar1 = *param_1;
  if (((int)((uint)*(ushort *)(iVar1 + 0x22) << 0x1e) < 0) &&
     (iVar2 = ie_find_in_frame(iVar1,(short)param_1[1],0,0), iVar2 != 0)) {
    iVar3 = (uint)*(byte *)((int)param_1 + 0x17) * 0x3b0 + DAT_00007140;
    iVar2 = fw_mem_equal(iVar2 + 2,*(undefined1 *)(iVar2 + 1),iVar3 + 0xf0,
                         *(undefined4 *)(iVar3 + 0xec));
    cVar4 = '\x01';
    if (iVar2 != 0) {
      uVar5 = u64_add_u32(0,*(undefined4 *)(iVar1 + 0x1c),*(undefined4 *)(iVar1 + 0x18));
      uVar6 = u64_add_u32(param_1[2],param_1[3],100);
      u64_cmp((int)uVar6,(int)((ulonglong)uVar6 >> 0x20),(int)uVar5,(int)((ulonglong)uVar5 >> 0x20))
      ;
      if (cVar4 == '\0') {
        return 1;
      }
    }
  }
  return 0;
}



/* ======================================================================
 * 00007144  lmc_req_enqueue_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000718a) */
/* WARNING: Removing unreachable block (ram,0x0000718a) */

undefined4 lmc_req_enqueue_dispatch(undefined1 *param_1)

{
  int iVar1;
  undefined4 uVar2;
  int iVar3;
  uint uVar4;
  
  if (0x17 < (byte)param_1[1]) {
    return 2;
  }
  iVar3 = (uint)(byte)param_1[1] * 0xa4 + DAT_000072a4;
  *(undefined1 *)(iVar3 + 0x398) = 0;
  iVar1 = DAT_000072a8;
  *(undefined1 *)(iVar3 + 0x399) = *param_1;
  *(undefined1 *)(iVar3 + 0x39a) = *(undefined1 *)(iVar1 + 10);
  fw_memcpy((void *)(iVar3 + 0x39c),param_1 + 4,0x28);
  *(undefined1 *)(iVar3 + 0x398) = 1;
  uVar4 = (uint)*(byte *)(iVar3 + 0x399);
                    /* WARNING: Could not recover jumptable at 0x0000718a. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0000718e <= uVar4) {
    uVar4 = (uint)DAT_0000718e;
  }
  uVar2 = (*(code *)((uint)*(byte *)(uVar4 + 0x718f) * 2 + 0x718f))
                    ((uint)*(byte *)(iVar1 + 10) * 0x3b0 + iVar1 + 0x18,DAT_000072ac);
  return uVar2;
}



/* ======================================================================
 * 00007290  lmc_req_slot_release
 * ====================================================================== */

void lmc_req_slot_release(int param_1)

{
  *(undefined1 *)(param_1 * 0xa4 + DAT_000072a4 + 0x398) = 0;
  return;
}



/* ======================================================================
 * 000072b0  ps_send_pending_poll_or_qosnull
 * ====================================================================== */

void ps_send_pending_poll_or_qosnull(int param_1)

{
  ushort uVar1;
  int iVar2;
  
  iVar2 = param_1 * 0x104 + DAT_00007448;
  if (*DAT_0000744c << 0x18 < 0) {
    *(ushort *)(iVar2 + 0x44) = *(ushort *)(iVar2 + 0x44) & 0xfffe;
    *(ushort *)(iVar2 + 0x46) = *(ushort *)(iVar2 + 0x46) | 1;
  }
  else {
    uVar1 = *(ushort *)(iVar2 + 0x5a);
    if ((int)((uint)uVar1 << 0x18) < 0) {
      *(ushort *)(iVar2 + 0x5a) = uVar1 & 0xff7f;
      if ((int)((uint)uVar1 << 0x1b) < 0) {
        tx_send_ps_poll();
        return;
      }
      tx_send_qos_null(param_1,*(undefined1 *)(iVar2 + 0x52));
      return;
    }
  }
  return;
}



/* ======================================================================
 * 000072f6  ps_timer_tick
 * ====================================================================== */

void ps_timer_tick(char *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  char cVar1;
  ushort uVar2;
  int iVar3;
  undefined4 uVar4;
  
  cVar1 = param_1[3];
  if ((int)((uint)*(ushort *)(param_1 + 0x1a) * 0x2000000) < 0) {
    if (param_1[0x13] == '\0') {
      iVar3 = *(int *)(param_1 + 0x20);
      *(int *)(param_1 + 0x20) = iVar3 + *(int *)(param_1 + 0x2c);
      if (*(uint *)(param_1 + 0x28) < (uint)(iVar3 + *(int *)(param_1 + 0x2c))) {
        *(uint *)(param_1 + 0x20) = *(uint *)(param_1 + 0x28);
      }
      if (*param_1 == '\x01') {
        if ((*(ushort *)(param_1 + 4) & 1) != 0) {
          if ((byte)param_1[2] < 3) {
            param_1[2] = param_1[2] + 1;
          }
          else {
            *(ushort *)(param_1 + 4) = *(ushort *)(param_1 + 4) & 0xfffe;
            event_send_ps_mode_error(cVar1,2,param_3,param_4,param_4);
          }
        }
      }
    }
    else {
      param_1[0x13] = '\0';
    }
    *(ushort *)(param_1 + 0x1a) = *(ushort *)(param_1 + 0x1a) | 0x80;
    if (*param_1 == '\x01') {
      ps_send_pending_poll_or_qosnull(cVar1);
    }
    uVar4 = *(undefined4 *)(param_1 + 0x20);
  }
  else {
    uVar2 = *(ushort *)(param_1 + 4);
    if ((uVar2 & 1) == 0) goto LAB_00007396;
    if (1 < (byte)param_1[2]) {
      *(ushort *)(param_1 + 4) = uVar2 & 0xfffe;
      event_send_ps_mode_error(cVar1,2,uVar2,param_4,param_4);
      goto LAB_00007396;
    }
    param_1[2] = param_1[2] + 1;
    *(ushort *)(param_1 + 0x1a) = *(ushort *)(param_1 + 0x1a) | 0x80;
    ps_send_pending_poll_or_qosnull(cVar1);
    uVar4 = DAT_00007450;
  }
  timer_start(param_1 + 0x58,uVar4);
LAB_00007396:
  ps_release_radio_if_all_idle(cVar1);
  return;
}



/* ======================================================================
 * 0000739e  ps_force_awake
 * ====================================================================== */

void ps_force_awake(int param_1)

{
  int iVar1;
  
  iVar1 = param_1 * 0x104 + DAT_00007448;
  *(undefined1 *)(iVar1 + 0x13e) = 1;
  *(byte *)(iVar1 + 0x58) = *(byte *)(iVar1 + 0x58) | 8;
  ps_reevaluate_all();
  return;
}



/* ======================================================================
 * 000073e2  ps_evaluate_all_vifs
 * ====================================================================== */

void ps_evaluate_all_vifs(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  iVar1 = DAT_00007448;
  if (*(byte *)(DAT_00007448 + 0x2a) < 2) {
    uVar3 = 0;
    *(undefined1 *)(DAT_00007448 + 0x2a) = 2;
    do {
      iVar2 = uVar3 * 0x104 + DAT_00007448;
      if ((*(char *)(iVar2 + 0x41) != '\0') ||
         (*(int *)(uVar3 * 0x3b0 + DAT_00007454 + 0x1c) << 0x1c < 0)) {
        if (*(char *)(iVar2 + 0x115) == '\0') {
          ps_schedule_next_tbtt_wake(uVar3);
          if (*(char *)(iVar2 + 0x115) != '\0') goto LAB_0000742e;
        }
        else {
LAB_0000742e:
          if (-1 < *DAT_0000744c << 0x13) goto LAB_0000743c;
        }
        *(undefined1 *)(iVar1 + 0x2a) = 3;
        ps_try_enter_sleep_all();
      }
LAB_0000743c:
      uVar3 = uVar3 + 1 & 0xff;
    } while (uVar3 < 2);
  }
  return;
}



/* ======================================================================
 * 00007458  pipe_find_or_alloc_upper
 * ====================================================================== */

short * pipe_find_or_alloc_upper(short *param_1)

{
  short *psVar1;
  short *psVar2;
  uint uVar3;
  
  psVar1 = (short *)0x0;
  uVar3 = 4;
  do {
    psVar2 = (short *)(uVar3 * 8 + DAT_00007550 + DAT_0000754c);
    if ((*(byte *)(psVar2 + 3) & 1) == 0) {
      if (psVar1 == (short *)0x0) goto LAB_000074aa;
    }
    else {
      if (((*psVar2 == *param_1) && (psVar2[1] == param_1[1])) && (psVar2[2] == param_1[2])) {
        if (*(char *)((int)psVar2 + 7) < '\0') {
          *(undefined1 *)((int)psVar2 + 7) = 0;
        }
        return psVar2;
      }
      if ((psVar1 == (short *)0x0) && (*(char *)((int)psVar2 + 7) == -0x19)) {
LAB_000074aa:
        psVar1 = psVar2;
      }
    }
    uVar3 = uVar3 + 1;
    if (7 < uVar3) {
      if (psVar1 == (short *)0x0) {
        return (short *)0x0;
      }
      *psVar1 = *param_1;
      psVar1[1] = param_1[1];
      psVar1[2] = param_1[2];
      *(undefined1 *)((int)psVar1 + 7) = 0;
      *(undefined1 *)(psVar1 + 3) = 5;
      return psVar1;
    }
  } while( true );
}



/* ======================================================================
 * 000074d0  pipe_find_or_alloc_lower
 * ====================================================================== */

short * pipe_find_or_alloc_lower(short *param_1)

{
  int iVar1;
  short *psVar2;
  short *psVar3;
  uint uVar4;
  
  iVar1 = DAT_00007554;
  psVar2 = (short *)0x0;
  uVar4 = 0;
  do {
    psVar3 = (short *)(uVar4 * 8 + DAT_00007550 + DAT_0000754c);
    if ((*(byte *)(psVar3 + 3) & 1) == 0) {
      if (psVar2 == (short *)0x0) {
        psVar2 = psVar3;
      }
    }
    else if (((*psVar3 == *param_1) && (psVar3[1] == param_1[1])) && (psVar3[2] == param_1[2])) {
      if (*(short *)(DAT_00007554 + 0x12) == 0) {
        return (short *)0x0;
      }
      if (*(char *)((int)psVar3 + 7) < '\0') {
        *(undefined1 *)((int)psVar3 + 7) = 0;
      }
      return psVar3;
    }
    uVar4 = uVar4 + 1;
    if (3 < uVar4) {
      if (psVar2 != (short *)0x0) {
        *psVar2 = *param_1;
        psVar2[1] = param_1[1];
        psVar2[2] = param_1[2];
        if (*(short *)(iVar1 + 0x12) != 0) {
          *(undefined1 *)((int)psVar2 + 7) = 0;
          *(undefined1 *)(psVar2 + 3) = 5;
          return psVar2;
        }
        *(undefined1 *)(psVar2 + 3) = 1;
      }
      return (short *)0x0;
    }
  } while( true );
}



/* ======================================================================
 * 000076d0  bab_evt_type5
 * ====================================================================== */

/* WARNING: Control flow encountered bad instruction data */

void bab_evt_type5(void)

{
                    /* WARNING: Bad instruction - Truncating control flow here */
  halt_baddata();
}



/* ======================================================================
 * 00007980  txp_should_defer_frame
 * ====================================================================== */

undefined4 txp_should_defer_frame(int param_1)

{
  undefined4 uVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  
  uVar1 = 1;
  if ((((*(byte *)(DAT_00007a60 + 0xd0) & 1) != 0) && (*(uint *)(param_1 + 0x18) < 0x25)) &&
     (iVar2 = *(int *)(param_1 + 0x1c), *(char *)(iVar2 + 0x18) == '\x03')) {
    if (*(char *)(iVar2 + 0x19) == '\0') {
      if (3 < *(byte *)(DAT_00007a60 + 0xd2)) {
        return 1;
      }
      uVar3 = (*(byte *)(iVar2 + 0x1b) & 0x3f) >> 2;
      uVar4 = (uint)*(byte *)(DAT_00007a58 + 0x1b) + uVar3 * 4 + DAT_00007a5c;
      if (((uint)*(ushort *)((uint)*(byte *)(param_1 + 0x2a) * 2 + DAT_00007a60 + 0xdc) & 1 << uVar3
          ) == 0) {
        return 1;
      }
      if ((uVar4 >> 1 & 1) == 0) {
        return 1;
      }
      *(char *)(iVar2 + 0x1b) = (char)uVar4;
      *(char *)(iVar2 + 0x1c) = (char)(uVar4 >> 8);
    }
    else {
      if (*(char *)(iVar2 + 0x19) != '\x02') {
        return 1;
      }
      if (-1 < (int)((uint)*(ushort *)(iVar2 + 0x1a) << 0x14)) {
        return 1;
      }
    }
    uVar1 = 0;
  }
  return uVar1;
}



/* ======================================================================
 * 00007b1c  mac_set_txop_limit
 * ====================================================================== */

void mac_set_txop_limit(undefined4 param_1)

{
  int iVar1;
  
  iVar1 = DAT_00007b58;
  *(undefined4 *)(DAT_00007b58 + 0x24) = param_1;
  *(uint *)(iVar1 + 0x1c) = (*(int *)(iVar1 + 0x38) - 1U & 0xffffff) + 0x40000000;
  return;
}



/* ======================================================================
 * 00007b34  mac_extend_txop_if_room
 * ====================================================================== */

void mac_extend_txop_if_room(uint param_1,int param_2)

{
  int iVar1;
  uint uVar2;
  
  iVar1 = DAT_00007b58;
  if (param_1 < 0x8001) {
    uVar2 = (*(int *)(DAT_00007b58 + 0x38) + 1) - param_2;
    if (uVar2 < param_1) {
      param_1 = param_1 - uVar2;
    }
    if (*(uint *)(DAT_00007b58 + 0x40) < param_1) {
      *(uint *)(DAT_00007b58 + 0x24) = param_1;
      *(uint *)(iVar1 + 0x1c) = (*(int *)(iVar1 + 0x38) - 1U & 0xffffff) + 0x40000000;
      return;
    }
  }
  return;
}



/* ======================================================================
 * 00007b5c  link_set_state
 * ====================================================================== */

void link_set_state(uint param_1,undefined1 param_2)

{
  if (param_1 < 8) {
    *(undefined1 *)(param_1 * 0x38 + DAT_00007e20 + 0x658) = param_2;
  }
  return;
}



/* ======================================================================
 * 00007b72  bab_process_ba_bitmap
 * ====================================================================== */

/* bab_process_ba_bitmap(link) -- process a received BlockAck against the
   outstanding A-MPDU.  Runs when link_set_state == 10.
   
   For each of the **16** subframe slots (`local_2c < 0x10`) held at
   g_txpipe[link*0x40 + 0x490 + i*4]:
   
     seq_delta = ctx->pas.wSeqCtl - ba->start_seq   (mod 0x1000)
     if (seq_delta >= 0x40)              -> outside the 64-bit window: free, flag 0x1000
     else if (!(ba_bitmap & (1 << seq_delta)))
           -> ACKED: free the descriptor, flag 0x1000
     else  -> NOT acked: bump the per-slot 4-bit retry nibble (saturating at 15),
              call pas_tx_retry_advance_mode0(); if it gives up, or the TID is no
              longer in the active mask at g_fw_ctx[if_id*2 + 0x648], set result
              0x0B and free; otherwise mark 0xFE (pending) and re-queue.
   
   Then: link_set_state(link, retried ? 8 : 5), and if the vif's active-TID mask is
   empty, link_set_state(link, 1).  Signals event 0x200000, plus 0x100000 unless
   hif_confirm_coalesce_hold() is holding confirms back.
   
   Two things worth noting for the throughput work:
   
   * **The 64-bit BA window is the real reorder bound**, and the 16 here is the same
     hardcoded A-MPDU subframe count as in txq_try_append_to_aggregate -- so 16 is
     enforced on both the build and the completion side, which makes patching only
     the build-side `cmp r0,#0x10` insufficient.  Anything above 16 would have no
     slot to be tracked in.
   * Retries are counted per subframe in a packed 4-bit nibble, and the counters at
     DAT_00007E2C+0xC / +0x10 are incremented on the requeue paths -- those are
     candidate telemetry for distinguishing "aggregates are fragmenting" from
     "frames are being retried" during a slow run. */

void bab_process_ba_bitmap(uint param_1)

{
  byte bVar1;
  ushort uVar2;
  short sVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  uint uVar7;
  undefined4 uVar8;
  int iVar9;
  int iVar10;
  int iVar11;
  int iVar12;
  char cVar13;
  char cVar14;
  undefined8 uVar15;
  uint local_2c;
  
  cVar13 = '\0';
  iVar9 = param_1 * 0x38 + DAT_00007e24;
  if (*(char *)(iVar9 + 0x10) == '\n') {
    iVar4 = param_1 * 0x40 + DAT_00007e28;
    bVar1 = *(byte *)(*(int *)(iVar4 + 0x490) + 0xf);
    iVar12 = (bVar1 & 7) << 2;
    local_2c = 0;
    do {
      iVar5 = iVar4 + local_2c * 4;
      iVar11 = *(int *)(iVar5 + 0x490);
      if (iVar11 != 0) {
        uVar2 = *(ushort *)(iVar9 + 0x16);
        if (*(ushort *)(iVar11 + 0x54) < uVar2) {
          sVar3 = 0x1000 - uVar2;
        }
        else {
          sVar3 = -uVar2;
        }
        sVar3 = *(ushort *)(iVar11 + 0x54) + sVar3;
        cVar14 = '\0';
        u64_cmp(sVar3,0,0x40,0);
        if (cVar14 == '\0') {
          uVar15 = u64_shl(1,0,sVar3);
          cVar14 = '\x01';
          u64_cmp((uint)uVar15 & *(uint *)(iVar9 + 0x18),
                  (uint)((ulonglong)uVar15 >> 0x20) & *(uint *)(iVar9 + 0x1c),0,0);
          if (cVar14 == '\0') {
            *(undefined4 *)(iVar5 + 0x490) = 0;
            *(byte *)(iVar11 + 0xf) = bVar1;
            *(uint *)(iVar11 + 0x2c) = *(uint *)(iVar11 + 0x2c) | 0x1000;
            tx_ctx_free_locked(iVar11);
          }
          else {
            iVar10 = (uint)(bVar1 >> 3) * 4 + iVar11;
            uVar6 = *(uint *)(iVar10 + 0x20);
            uVar7 = uVar6 >> iVar12 & 0xf;
            *(uint *)(iVar10 + 0x20) = (uVar7 < 0xf) + uVar7 << iVar12 | uVar6 & ~(0xf << iVar12);
            *(byte *)(iVar11 + 0xf) = bVar1;
            *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) & 0xffffffdf;
            iVar10 = pas_tx_retry_advance_mode0(iVar11);
            *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) & 0xfff7ffff;
            if ((iVar10 == 0) ||
               (((uint)*(ushort *)((uint)*(byte *)(iVar11 + 0x69) * 2 + DAT_00007e20 + 0x648) &
                1 << *(sbyte *)(iVar11 + 0x52)) == 0)) {
              *(undefined2 *)(iVar11 + 0x1c) = 0xb;
              *(uint *)(iVar11 + 0x2c) = *(uint *)(iVar11 + 0x2c) | 0x2000;
              tx_ctx_free_locked(iVar11);
              *(undefined4 *)(iVar5 + 0x490) = 0;
            }
            else {
              *(int *)(DAT_00007e2c + 0xc) = *(int *)(DAT_00007e2c + 0xc) + 1;
LAB_00007d50:
              *(undefined2 *)(iVar11 + 0x1c) = 0xfe;
              cVar13 = cVar13 + '\x01';
              *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) | DAT_00007e30;
            }
          }
        }
        else {
          iVar10 = (uint)(bVar1 >> 3) * 4 + iVar11;
          uVar6 = *(uint *)(iVar10 + 0x20);
          uVar7 = uVar6 >> iVar12 & 0xf;
          *(uint *)(iVar10 + 0x20) = (uVar7 < 0xf) + uVar7 << iVar12 | uVar6 & ~(0xf << iVar12);
          *(byte *)(iVar11 + 0xf) = bVar1;
          *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) & 0xffffffdf;
          iVar10 = pas_tx_retry_advance_mode0(iVar11);
          *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) & 0xfff7ffff;
          if (iVar10 != 0) {
            uVar6 = (uint)*(byte *)(iVar11 + 0x69) * 2 + DAT_00007e20;
            uVar7 = (uint)*(byte *)(iVar11 + 0x52);
            cVar14 = uVar7 == 0 && 0xfffff9bf < uVar6 ||
                     uVar7 != 0 && (1 << uVar7 - 1 & 0x80000000U) != 0;
            if (((uint)*(ushort *)(uVar6 + 0x648) & 1 << uVar7) != 0) {
              u64_cmp(sVar3,0,0x50,0);
              if (cVar14 == '\0') {
                *(int *)(DAT_00007e2c + 0x10) = *(int *)(DAT_00007e2c + 0x10) + 1;
              }
              goto LAB_00007d50;
            }
          }
          *(undefined2 *)(iVar11 + 0x1c) = 0xb;
          *(uint *)(iVar11 + 0x2c) = *(uint *)(iVar11 + 0x2c) | 0x2000;
          tx_ctx_free_locked(iVar11);
          *(undefined4 *)(iVar5 + 0x490) = 0;
        }
      }
      local_2c = local_2c + 1 & 0xff;
    } while (local_2c < 0x10);
    if (cVar13 == '\0') {
      uVar8 = 5;
    }
    else {
      uVar8 = 8;
    }
    link_set_state(param_1,uVar8);
    if (*(short *)((uint)*(byte *)(iVar9 + 0x2f) * 2 + DAT_00007e20 + 0x648) == 0) {
      link_set_state(param_1,1);
      *(ushort *)(DAT_00007e34 + 0x18) =
           *(ushort *)(DAT_00007e34 + 0x18) & ~(ushort)(1 << (param_1 & 0xff));
    }
    evt_flags_set(DAT_00007e38,0x200000);
    iVar9 = hif_confirm_coalesce_hold();
    if (iVar9 == 0) {
      evt_flags_set(DAT_00007e38,0x100000);
    }
  }
  return;
}



/* ======================================================================
 * 00007dce  txpipe_find_by_mac_tid
 * ====================================================================== */

byte txpipe_find_by_mac_tid(uint param_1,uint param_2,short *param_3)

{
  char *pcVar1;
  byte bVar2;
  
  pcVar1 = (char *)(DAT_00007e24 + 0x10);
  bVar2 = 0;
  while( true ) {
    if (*(byte *)(DAT_00007e24 + 4) <= bVar2) {
      return 8;
    }
    if ((*pcVar1 != '\0') &&
       (((param_3 == (short *)0x0 ||
         (((*param_3 == *(short *)(pcVar1 + 0x18) && (param_3[1] == *(short *)(pcVar1 + 0x1a))) &&
          (param_3[2] == *(short *)(pcVar1 + 0x1c))))) &&
        (((byte)pcVar1[0x1e] == param_2 && ((byte)pcVar1[0x1f] == param_1)))))) break;
    pcVar1 = pcVar1 + 0x38;
    bVar2 = bVar2 + 1;
  }
  return bVar2;
}



/* ======================================================================
 * 00007e50  phy_rx_disable_and_drain
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void phy_rx_disable_and_drain(void)

{
  uint *puVar1;
  int iVar2;
  
  puVar1 = _DAT_00007ef4;
  if ((*_DAT_00007ef4 & 1) != 0) {
    *_DAT_00007ef4 = *_DAT_00007ef4 & 0xfffffffe;
    iVar2 = _DAT_0ac00004 + 0x400000;
    while ((int)(*puVar1 << 8) < 0) {
      if (-1 < _DAT_0ac00004 - iVar2) {
        fw_assert(s_pac_phy_c_00007ef7 + 1,0xe2,3);
      }
    }
  }
  *(byte *)(DAT_00007f04 + 0x15) = *(byte *)(DAT_00007f04 + 0x15) | 1;
  return;
}



/* ======================================================================
 * 00007e90  phy_rx_enable
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void phy_rx_enable(void)

{
  if ((*_DAT_00007ef4 & 1) == 0) {
    *_DAT_00007ef4 = *_DAT_00007ef4 | 1;
  }
  *(byte *)(DAT_00007f04 + 0x15) = *(byte *)(DAT_00007f04 + 0x15) & 0xfe;
  return;
}



/* ======================================================================
 * 00007eaa  pac_phy_calc_duration
 * ====================================================================== */

uint pac_phy_calc_duration(int param_1,int param_2)

{
  int iVar1;
  uint uVar2;
  
  if (7 < param_1 - 0xeU) {
    fw_assert(s_pac_phy_c_00007ef7 + 1,0x10e,4);
  }
  iVar1 = __udivsi3(param_2 * 8 + *(int *)(param_1 * 4 + DAT_00007f08 + -0x38) + 0x15);
  uVar2 = iVar1 * 3 + 9;
  if (DAT_00007f0c < uVar2) {
    fw_assert(s_pac_phy_c_00007ef7 + 1,0x117,5);
  }
  return uVar2;
}



/* ======================================================================
 * 00007f10  pac_phy_start_op
 * ====================================================================== */

void pac_phy_start_op(undefined1 param_1)

{
  int iVar1;
  
  iVar1 = DAT_000082bc;
  *(undefined1 *)(DAT_000082bc + 0x10) = param_1;
  *(undefined1 *)(iVar1 + 0x21) = 0;
  phy_state_cmd_dispatch(iVar1 + 0x10,iVar1 + 0x18);
  *(uint *)(iVar1 + 0xc) = (uint)*(byte *)(iVar1 + 0x18);
  if (*(int *)(iVar1 + 0x1c) != 0) {
    timer_start(iVar1 + -8);
  }
  return;
}



/* ======================================================================
 * 00007f40  pas_reprogram_all_vif_rate_tables
 * ====================================================================== */

void pas_reprogram_all_vif_rate_tables(void)

{
  bool bVar1;
  bool bVar2;
  int iVar3;
  void *pvVar4;
  undefined4 uVar5;
  uint uVar6;
  
  iVar3 = DAT_000082c0;
  bVar1 = false;
  bVar2 = false;
  uVar6 = 0;
  do {
    pvVar4 = PTR_g_pas_ctx_000082c4;
    if (*(char *)((int)PTR_g_pas_ctx_000082c4 + uVar6 * 0x98 + 0x470) == '\x02') {
      *(undefined1 *)((int)PTR_g_pas_ctx_000082c4 + uVar6 * 0x98 + 0x491) = 0;
      if (bVar1) {
        if (bVar2) goto LAB_00007f8a;
        bVar2 = true;
        *(undefined1 *)((int)pvVar4 + uVar6 * 0x98 + 0x491) = 1;
        if (*(char *)((int)pvVar4 + uVar6 * 0x98 + 0x473) == '\0') {
          uVar5 = 0x1000000;
        }
        else {
          uVar5 = 0x4000000;
        }
        *(undefined4 *)(iVar3 + 0x30) = uVar5;
      }
      else {
        bVar1 = true;
      }
      pas_program_rate_tables();
    }
LAB_00007f8a:
    uVar6 = uVar6 + 1;
    if (2 < uVar6) {
      if (bVar2) {
        *(undefined4 *)(DAT_000082c0 + 0xd4) = 0x1000000;
        return;
      }
      *(undefined4 *)(DAT_000082c0 + 0xd4) = 0;
      return;
    }
  } while( true );
}



/* ======================================================================
 * 00007fa6  pas_compute_tx_timing
 * ====================================================================== */

/* pas_compute_tx_timing(pas) -- recompute per-frame TX durations.
   
   *** BASE CORRECTION: `pas` IS tx_ctx + 0x54, NOT tx_ctx. ***
   Every offset below is relative to that nested sub-struct.  Proof:
     - tx_frame_done_release (0xD1C4) calls pas_retime_and_kick(param_1 + 0x54)
       and tx_ctx_free_locked(param_1 + 0x54);
     - pas+0x69 is asserted <= 1 and indexes a 0x3B0 stride (= sizeof xr_vif), so
       it is IF_ID -- and 0x54 + 0x69 = 0xBD, exactly the tx_ctx byte that
       tx_lmac_req_submit writes if_id into.
   The whole pas_* / txop_budget_* / txq_try_append family shares this view.
   
   Offset map (pas -> tx_ctx):
     pas+0x04 = ctx+0x58  flags      pas+0x0C = ctx+0x60  AC
     pas+0x0F = ctx+0x63  rate idx   pas+0x1E = ctx+0x72  try count
     pas+0x48 = ctx+0x9C  AIRTIME    pas+0x69 = ctx+0xBD  if_id
     pas+0x6C = ctx+0xC0  TID/pipe
   
   Writes param_1[0x12] = pas+0x48 = **tx_ctx+0x9C** = sum of the preamble and
   payload durations from airtime_compute().  Earlier docs called this field
   "tx_ctx+0x48"; that address is a DIFFERENT field -- tx_classify_hdr_len
   (0xE3C6) writes the payload BYTE length there.  The substance of the earlier
   finding (the budget unit is airtime in us, not bytes) is unaffected.
   
   Also note: the 0x98-stride array at g_fw_ctx is indexed by IF_ID here (max 3
   entries), not by link id.  Fields +0x472 (PHY mode), +0x478 (basic_rate_set),
   +0x4AC (per-AC try count), +0x4E0 (per-TID aggregate airtime budget) are
   therefore PER-VIF, not per-link. */

void pas_compute_tx_timing(xr_tx_pas *pas)

{
  char cVar1;
  int iVar2;
  void *pvVar3;
  byte bVar4;
  ushort uVar5;
  int iVar6;
  undefined4 uVar7;
  uint uVar8;
  uint uVar9;
  
  pvVar3 = PTR_g_pas_ctx_000082c4;
  iVar2 = DAT_000082c8 + (uint)pas->bIfId * 0x98;
  uVar9 = pas->dwFlags & 0xc00;
  if (uVar9 == 0) {
    pas->wDurPre = 0;
    pas->wDurPreB = 0;
    pas->bHwRateCode = 0xff;
  }
  else {
    if (1 < pas->bIfId) {
      fw_assert(s_pas_c_000082cc,0x49a,1000);
    }
    iVar6 = (uint)pas->bIfId * 0x3b0 + DAT_000082d4;
    if ((int)(uVar9 << 0x14) < 0) {
      bVar4 = *(byte *)(iVar6 + 0x12d);
      pas->bHwRateCode = bVar4;
      if (bVar4 == 0xff) {
        bVar4 = *(byte *)(iVar6 + 0x25);
LAB_0000801c:
        pas->bHwRateCode = bVar4;
      }
    }
    else {
      bVar4 = *(byte *)(iVar6 + 300);
      pas->bHwRateCode = bVar4;
      if (bVar4 == 0xff) {
        bVar4 = *(byte *)((int)pvVar3 + (uint)pas->bRateIdx + iVar2);
        goto LAB_0000801c;
      }
    }
    if ((int)(uVar9 << 0x15) < 0) {
      uVar7 = 0x14;
      pas->wDurPreB =
           *(ushort *)
            ((uint)*(byte *)((int)pvVar3 + (uint)pas->bHwRateCode + iVar2) * 2 + DAT_000082d8 + 0x48
            );
    }
    else {
      uVar7 = 0xe;
      pas->wDurPreB = 0;
    }
    airtime_compute(&pas->wDurPreA,&pas->wDurPre,pas->bHwRateCode,*(undefined2 *)(DAT_000082d8 + 2),
                    uVar7,pas->dwFlags & 8);
  }
  iVar6 = DAT_000082d8;
  airtime_compute(&pas->wDurPayB,&pas->wDurPayA,pas->bRateIdx,*(undefined2 *)(DAT_000082d8 + 2),
                  pas->wFrameLen + 4,pas->dwFlags & 8);
  pas->wDurAck = *(ushort *)
                  ((uint)*(byte *)((int)pvVar3 + (uint)pas->bRateIdx + iVar2) * 2 + iVar6 + 0x48);
  if ((int)(pas->dwFlags << 0x16) < 0) {
    uVar5 = 0;
  }
  else {
    if (-1 < (int)(pas->dwFlags << 0x11)) {
      uVar9 = (uint)pas->bIfId;
      cVar1 = *(char *)((int)PTR_g_pas_ctx_000082c4 + uVar9 * 0x98 + 0x472);
      if ((cVar1 == '\x06') || (cVar1 == '\x05')) {
        uVar8 = pas->dwHdr;
        if ((*(short *)((int)PTR_g_pas_ctx_000082c4 + uVar9 * 6 + 0x460) == *(short *)(uVar8 + 10))
           && ((*(short *)((int)PTR_g_pas_ctx_000082c4 + uVar9 * 6 + 0x462) ==
                *(short *)(uVar8 + 0xc) &&
               (*(short *)((int)PTR_g_pas_ctx_000082c4 + uVar9 * 6 + 0x464) ==
                *(short *)(uVar8 + 0xe))))) {
          pas->bFrameKind = 0xe;
          goto LAB_000080ee;
        }
      }
      pas->bFrameKind = 0x11;
      goto LAB_000080ee;
    }
    pas->bFrameKind = 0xc;
    uVar5 = *(ushort *)
             ((uint)*(byte *)((int)pvVar3 + (uint)pas->bRateIdx + iVar2) * 2 + iVar6 + 0x74);
  }
  pas->wDurAck = uVar5;
LAB_000080ee:
  if (pas->wDurPre == 0) {
    uVar9 = (uint)pas->wDurPayB;
    uVar8 = (uint)pas->wDurAck;
  }
  else {
    uVar9 = (uint)pas->wDurPreA + (uint)pas->wDurPreB;
    uVar8 = (uint)pas->wDurPayA + (uint)pas->wDurAck;
  }
  pas->dwAirtimeUs = uVar9 + uVar8;
  return;
}



/* ======================================================================
 * 0000810c  pas_txq_compact_and_push
 * ====================================================================== */

void pas_txq_compact_and_push(uint *param_1,uint param_2)

{
  uint uVar1;
  uint uVar2;
  uint uVar3;
  
  uVar3 = *param_1 & 0xff;
  uVar1 = param_1[1] & 0xff;
  uVar2 = uVar1;
  if (uVar3 != uVar1) {
    for (; uVar3 != uVar1; uVar3 = uVar3 + 1 & 0x3f) {
      if (param_1[uVar3 + 2] != 0) {
        param_1[uVar2 + 2] = param_1[uVar3 + 2];
        uVar2 = uVar2 + 1 & 0x3f;
        param_1[uVar3 + 2] = 0;
      }
    }
    *param_1 = uVar1;
    param_1[1] = uVar2;
    uVar3 = uVar1;
  }
  if (*(char *)(param_2 + 0x53) != '\0') {
    if (*(char *)(param_2 + 0x53) == '\x01') {
      uVar1 = uVar2 + 1 & 0x3f;
      param_1[uVar2 + 2] = param_2;
      param_1[1] = uVar1;
      if (uVar1 != uVar3) {
        return;
      }
      uVar3 = 7;
      uVar2 = DAT_000082c8 + 0xba;
    }
    else {
      uVar3 = 8;
      uVar2 = DAT_000082c8 + 0xbe;
    }
    fw_assert(s_pas_c_000082cc,uVar2,uVar3);
    return;
  }
  uVar3 = uVar3 - 1 & 0x3f;
  if (uVar3 == uVar2) {
    fw_assert(s_pas_c_000082cc,DAT_000082c8 + 0xaa,6);
  }
  param_1[uVar3 + 2] = param_2;
  *param_1 = uVar3;
  return;
}



/* ======================================================================
 * 000081a6  pas_txq_push_global
 * ====================================================================== */

void pas_txq_push_global(uint param_1)

{
  uint *puVar1;
  uint uVar2;
  uint uVar3;
  uint uVar4;
  
  puVar1 = DAT_000082dc;
  uVar4 = *DAT_000082dc & 0xff;
  uVar2 = DAT_000082dc[1] & 0xff;
  uVar3 = uVar2;
  if (uVar4 != uVar2) {
    for (; uVar4 != uVar2; uVar4 = uVar4 + 1 & 0x3f) {
      if (puVar1[uVar4 + 2] != 0) {
        puVar1[uVar3 + 2] = puVar1[uVar4 + 2];
        uVar3 = uVar3 + 1 & 0x3f;
        puVar1[uVar4 + 2] = 0;
      }
    }
    *puVar1 = uVar2;
    puVar1[1] = uVar3;
    uVar4 = uVar2;
  }
  if (*(char *)(param_1 + 0x53) != '\0') {
    if (*(char *)(param_1 + 0x53) == '\x01') {
      uVar2 = uVar3 + 1 & 0x3f;
      puVar1[uVar3 + 2] = param_1;
      puVar1[1] = uVar2;
      if (uVar2 != uVar4) {
        return;
      }
      uVar4 = 7;
      uVar3 = DAT_000082c8 + 0xba;
    }
    else {
      uVar4 = 8;
      uVar3 = DAT_000082c8 + 0xbe;
    }
    fw_assert(s_pas_c_000082cc,uVar3,uVar4);
    return;
  }
  uVar4 = uVar4 - 1 & 0x3f;
  if (uVar4 == uVar3) {
    fw_assert(s_pas_c_000082cc,DAT_000082c8 + 0xaa,6);
  }
  puVar1[uVar4 + 2] = param_1;
  *puVar1 = uVar4;
  return;
}



/* ======================================================================
 * 000081ac  pas_retime_and_kick
 * ====================================================================== */

void pas_retime_and_kick(xr_tx_pas *param_1)

{
  param_1->bFrameKind = 0xff;
  pas_compute_tx_timing(param_1);
  pas_txq_push_global(param_1);
  return;
}



/* ======================================================================
 * 000081c4  mac_arm_beacon_tx
 * ====================================================================== */

void mac_arm_beacon_tx(void)

{
  *(undefined4 *)(DAT_000082e0 + 0x28) = 5;
  if (*(short *)(DAT_000082e4 + 0x12) != 0) {
    *(undefined2 *)(DAT_000082d8 + 8) = 0x2000;
    mac_program_beacon_timer();
  }
  *DAT_000082e8 = *DAT_000082e8 | 0x1000000;
  return;
}



/* ======================================================================
 * 000081ee  pas_latch_vif_slot_flag
 * ====================================================================== */

void pas_latch_vif_slot_flag(int param_1)

{
  *(byte *)(param_1 + 0x6a) =
       *(byte *)((int)PTR_g_pas_ctx_000082c4 + (uint)*(byte *)(param_1 + 0x69) * 0x98 + 0x471) & 1;
  return;
}



/* ======================================================================
 * 0000820a  phy_state_advance
 * ====================================================================== */

void phy_state_advance(undefined1 param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  iVar3 = DAT_000082e4;
  iVar1 = DAT_000082d8;
  *(undefined1 *)(DAT_000082d8 + 0xb) = 0;
  iVar2 = DAT_000082e0;
  if (*(char *)(iVar3 + 0x16) == '\x01') {
    *(undefined1 *)(iVar1 + 1) = param_1;
    *(undefined1 *)(iVar3 + 0x16) = 2;
    iVar1 = DAT_000082e8;
    *(undefined1 *)(iVar2 + 0x5c) = 0;
    evt_flags_set((uint *)(iVar1 + 4),0x40000);
    phy_wake_sequence();
  }
  else {
    if (*(char *)(DAT_000082e0 + 0x5d) != '\0') {
      mac_reprogram_after_channel();
    }
    *(undefined1 *)(iVar1 + 1) = 0;
    if (*(char *)(iVar3 + 0x16) == '\x03') {
      phy_resume_state4();
      return;
    }
  }
  return;
}



/* ======================================================================
 * 0000824e  txop_budget_reload
 * ====================================================================== */

void txop_budget_reload(void)

{
  void *pvVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  if (*(int *)(DAT_000082e4 + 0x70) != 0) {
    irq_fiq_disable_save();
    pvVar1 = PTR_g_pas_ctx_000082c4;
    uVar2 = 0;
    do {
      iVar3 = uVar2 * 0xc;
      iVar4 = uVar2 * 4;
      uVar2 = uVar2 + 1 & 0xff;
      *(uint *)((int)pvVar1 + iVar4 + 0x440) = (uint)*(ushort *)((int)pvVar1 + iVar3 + 0x408);
    } while (uVar2 < 4);
    irq_fiq_restore();
  }
  return;
}



/* ======================================================================
 * 0000828c  txop_budget_check
 * ====================================================================== */

/* txop_budget_check(pas) -- per-AC airtime budget gate.
   Takes the tx_ctx+0x54 sub-struct (xr_tx_pas), not tx_ctx.
   Returns 0 = over budget (caller closes the aggregate), 1 = ok.
   
     base  = g_fw_ctx + ac_map[pas->bAc]*4
     limit = *(int *)(base + 0x430)
     used  = *(int *)(base + 0x440)
     if (limit != 0 && (limit - used) < pas->dwAirtimeUs) return 0;
     return 1;
   
   `limit == 0` means unlimited.  txop_budget_consume (0x000082F0) adds
   pas->dwAirtimeUs to `used`; txop_budget_reload (0x0000824E) reloads `used` from
   the u16 at base+0x408 for all 4 ACs under IRQ lock.
   
   *** THERE ARE TWO SEPARATE AIRTIME BUDGETS.  THIS IS THE ONE MAINLINE DISABLES. ***
   Corrected after finding edca_apply_queue_params (0x0001364C):
   
     this one, +0x430 : written as `allowedMediumTime * 32` from
                       **WSM 0x0012 SET_TX_QUEUE_PARAMS** (payload +8).
                       cw1200 sends WSM_TX_QUEUE_SET(..., 0, 0, 0) for all four
                       queues -- allowedMediumTime = 0 -- so **this budget is
                       unconditionally off on mainline**, and the reload timer does
                       not even start (edca_apply_queue_params only arms it when at
                       least one queue is non-zero).
   
     the other, +0x4E0 : the EDCA `txop_limit`, per-AC, in the per-vif 0x98 block,
                       written from **WSM 0x0013 SET_EDCA_PARAMS** and read by
                       txq_agg_airtime_budget (0x0000A05E).  cw1200 DOES program
                       this from mac80211's params->txop (already multiplied by
                       TXOP_UNIT 32), so it is live whenever the AP advertises a
                       non-zero TXOP limit for the AC.
   
   An earlier revision of this comment said this +0x430 limit "comes straight from
   the EDCA params" and that cw1200_conf_tx overwrites it from the AP's WMM IE.
   That described the *other* budget.  The AP's WMM TXOP reaches +0x4E0, never here.
   
   UNITS are unchanged and still verified: pas->dwAirtimeUs (= tx_ctx+0x9C) is an
   airtime estimate in microseconds from airtime_compute(), not a byte count. */

int txop_budget_check(xr_tx_pas *pas)

{
  byte bVar1;
  int iVar2;
  
  bVar1 = *(byte *)(DAT_000082ec + (uint)pas->bAc);
  iVar2 = *(int *)((int)PTR_g_pas_ctx_000082c4 + (uint)bVar1 * 4 + 0x430);
  if ((iVar2 != 0) &&
     ((uint)(iVar2 - *(int *)((int)PTR_g_pas_ctx_000082c4 + (uint)bVar1 * 4 + 0x440)) <
      pas->dwAirtimeUs)) {
    return 0;
  }
  return 1;
}



/* ======================================================================
 * 000082f0  txop_budget_consume
 * ====================================================================== */

void txop_budget_consume(int param_1)

{
  int iVar1;
  int *piVar2;
  
  iVar1 = (uint)*(byte *)(DAT_00008338 + (uint)*(byte *)(param_1 + 0xc)) * 4 + DAT_0000833c;
  if (*(int *)(iVar1 + 0x430) != 0) {
    piVar2 = (int *)(iVar1 + 0x440);
    *piVar2 = *piVar2 + *(int *)(param_1 + 0x48);
  }
  return;
}



/* ======================================================================
 * 00008328  tx_submit_wrapper
 * ====================================================================== */

void tx_submit_wrapper(int param_1)

{
  txp_submit_to_pipe(*(undefined4 *)(param_1 + 0x4c),param_1,*(undefined2 *)(param_1 + 0x36));
  return;
}



/* ======================================================================
 * 00008348  pas_rate_to_hw_code
 * ====================================================================== */

undefined1 pas_rate_to_hw_code(int param_1)

{
  return *(undefined1 *)(DAT_00008724 + param_1);
}



/* ======================================================================
 * 0000834e  pas_build_phy_rate_words
 * ====================================================================== */

void pas_build_phy_rate_words
               (undefined4 *param_1,uint *param_2,uint param_3,int param_4,uint param_5)

{
  uint uVar1;
  
  *param_1 = 2;
  if (param_3 < 4) {
    if (((*(char *)(DAT_00008728 + 5) == '\0') || (param_3 == 0)) ||
       ((*(char *)(DAT_00008728 + 5) == '\x02' && (param_3 == 1)))) {
      uVar1 = 0x400;
    }
    else {
      uVar1 = 0;
    }
  }
  else if (param_3 < 0xe) {
    uVar1 = 0x800;
  }
  else {
    if (param_4 << 0x1a < 0) {
      *param_1 = 6;
    }
    if (param_4 << 0x1c < 0) {
      uVar1 = 0x1400;
    }
    else {
      uVar1 = 0x1000;
    }
  }
  *param_2 = uVar1;
  *param_2 = *(byte *)(DAT_00008724 + -0x16 + param_3) & 0xf | *param_2 | (param_5 & 7) << 0x10;
  return;
}



/* ======================================================================
 * 000083b2  pas_rate_phy_class
 * ====================================================================== */

undefined4 pas_rate_phy_class(int param_1,uint param_2,int param_3)

{
  undefined4 uVar1;
  
  uVar1 = 1;
  if (param_2 < 4) {
    if (((-1 < param_1 << 0x17) && (param_2 != 0)) && ((-1 < param_1 << 0x15 || (param_2 != 1)))) {
      uVar1 = 0;
    }
  }
  else if (param_2 < 0xe) {
    uVar1 = 2;
  }
  else if (param_3 == 0) {
    uVar1 = 4;
  }
  else {
    uVar1 = 5;
  }
  return uVar1;
}



/* ======================================================================
 * 000083e2  airtime_compute
 * ====================================================================== */

void airtime_compute(short *param_1,short *param_2,uint param_3,int param_4,int param_5,
                    undefined4 param_6)

{
  short sVar1;
  uint uVar2;
  
  uVar2 = pas_rate_phy_class(param_4,param_3,param_6);
  if (uVar2 < 2) {
    sVar1 = __udivsi3(param_5 * 0x10 + (uint)*(ushort *)(DAT_00008724 + -0x66 + (param_3 & 3) * 2) +
                      -1);
    if (uVar2 == 1) {
      sVar1 = sVar1 + 0xc0;
    }
    else {
      sVar1 = sVar1 + 0x60;
    }
  }
  else {
    sVar1 = __udivsi3((param_5 * 8 + 0x16U & 0xffff) +
                      (uint)*(ushort *)(DAT_00008724 + -0x4e + (param_3 - 6 & 0xf) * 2) + -1);
    sVar1 = sVar1 * 4;
    if (param_4 << 0x1b < 0) {
      sVar1 = sVar1 + 6;
    }
    if (uVar2 == 2) {
      sVar1 = sVar1 + 0x14;
      if (param_4 << 0x19 < 0) {
        sVar1 = sVar1 * 2;
      }
    }
    else if (uVar2 == 5) {
      sVar1 = sVar1 + 0x24;
    }
    else {
      sVar1 = sVar1 + 0x18;
    }
  }
  if (param_1 != (short *)0x0) {
    *param_1 = sVar1;
  }
  if (param_2 != (short *)0x0) {
    if (param_4 << 0x19 < 0) {
      sVar1 = sVar1 + 0x20;
    }
    else if ((param_4 << 0x1b < 0) && (uVar2 < 3)) {
      sVar1 = sVar1 + 10;
    }
    else {
      sVar1 = sVar1 + 0x10;
    }
    *param_2 = sVar1;
  }
  return;
}



/* ======================================================================
 * 00008486  pas_build_rate_entry
 * ====================================================================== */

void pas_build_rate_entry(int param_1,uint param_2,int param_3,int param_4,uint *param_5)

{
  uint uVar1;
  ushort local_2c [2];
  undefined4 local_28;
  int iStack_24;
  uint uStack_20;
  int iStack_1c;
  int local_18;
  
  local_28 = 1;
  iStack_24 = param_1;
  uStack_20 = param_2;
  iStack_1c = param_3;
  local_18 = param_4;
  airtime_compute(0,local_2c,param_3,param_1);
  *param_5 = (uint)local_2c[0] | local_18 << 0x18;
  airtime_compute(0,local_2c,param_3,param_1);
  uVar1 = (uint)local_2c[0];
  if (param_2 < 3) {
    if ((param_3 == 0) || ((param_3 == 1 && (param_1 << 0x15 < 0)))) {
      param_2 = 1;
    }
  }
  else {
    param_2 = pas_rate_phy_class(param_1,param_3,local_28);
  }
  param_5[1] = param_2 << 0x1c | uVar1 | (uint)*(byte *)(DAT_00008724 + -0x16 + param_3) << 0x18;
  uVar1 = ofdm_calc_duration(param_3,0xe);
  param_5[2] = uVar1 | uVar1 << 0xc;
  uVar1 = ofdm_calc_duration(param_3,0x20);
  param_5[3] = uVar1 | uVar1 << 0xc;
  return;
}



/* ======================================================================
 * 00008520  pas_build_rate_tables
 * ====================================================================== */

/* pas_build_rate_tables(pas_vif) -- pas_vif = g_fw_ctx + if_id*0x98 + 0x470.
   
   *** THIS IS THE basic_rate_set READER. *** Question closed.
   
     u32 brt = *(u32 *)(pas_vif + 8);   /* == g_fw_ctx + if_id*0x98 + 0x478,
                                           written by vif_set_basic_rates from
                                           WSM_START +0x30 / JOIN basic_rate_set */
   
   brt is a rate bitmap over the 22-entry rate index space (0..3 CCK, 6..12
   OFDM, 14..21 MCS0..7).  Two gap-fill loops run FIRST:
   
     for (i = 0; i < 4;  i++) { if (brt & 1<<i) break; brt |= (1<<i) & 0x00F; }
     for (i = 6; i < 11; i++) { if (brt & 1<<i) break; brt |= (1<<i) & 0x540; }
   
   *** CONSEQUENCE: brt == 0 IS SELF-HEALING. *** With brt = 0 neither loop ever
   breaks, so it becomes 0x00F | 0x540 = 0x54F = all four CCK rates plus OFDM
   6/12/24 Mbit/s -- precisely the correct default basic rate set.
   
   That settles the `brt: 0x00000000` bug recorded in PROGRESS.md: the value
   reaches live state AND has a real reader, but a zero is replaced by a sane
   default before use.  The driver fix is COSMETIC, not functional.
   
   The body then walks rate indices 0..0x15 and, for each, records the highest
   brt-set rate at or below it:
   
     pas_vif[0x24 + idx]      = fallback/basic rate for rate index idx
     g_fw_ctx[0x48 + idx*2]   = airtime for a 14-byte frame at that rate
     g_fw_ctx[0x74 + idx*2]   = airtime for a 32-byte frame at that rate
     g_fw_ctx[0x46C + idx]    = PHY class byte (pas_rate_phy_class)
     g_fw_ctx[0x47A + idx]    = second-stream fallback (MCS range only)
   
   pas_compute_tx_timing reads the +0x48 / +0x74 tables back, so brt feeds the
   airtime numbers that the TXOP budget is measured against. */

void pas_build_rate_tables(int param_1)

{
  byte bVar1;
  undefined2 uVar2;
  int iVar3;
  undefined1 uVar4;
  uint uVar5;
  uint uVar6;
  int iVar7;
  uint uVar8;
  int iVar9;
  uint uVar10;
  byte bVar11;
  
  uVar10 = *(uint *)(param_1 + 8);
  uVar2 = *(undefined2 *)(DAT_00008728 + 2);
  uVar6 = 0;
  do {
    uVar8 = 1 << (uVar6 & 0xff);
    if ((uVar10 & uVar8) != 0) break;
    uVar10 = uVar10 | uVar8 & 0xf;
    uVar6 = uVar6 + 1;
  } while (uVar6 < 4);
  uVar6 = 6;
  do {
    uVar8 = 1 << (uVar6 & 0xff);
    if ((uVar10 & uVar8) != 0) break;
    uVar10 = uVar10 | uVar8 & 0x540;
    uVar6 = uVar6 + 1;
  } while (uVar6 < 0xb);
  uVar6 = 0;
  uVar8 = 0;
  do {
    uVar5 = 1 << (uVar8 & 0xff);
    if ((uVar5 & DAT_0000872c) != 0) {
      if ((uVar5 & uVar10) != 0) {
        uVar6 = uVar8;
      }
      uVar5 = uVar6 & 0xff;
      *(char *)(param_1 + uVar8 + 0x24) = (char)uVar6;
      iVar7 = uVar8 * 2 + DAT_00008728;
      airtime_compute(0,iVar7 + 0x48,uVar5,uVar2,0xe,1);
      airtime_compute(0,iVar7 + 0x74,uVar5,uVar2,0x20,1);
      uVar4 = pas_rate_phy_class(uVar2,uVar5,1);
      *(undefined1 *)(DAT_00008728 + uVar8 + 0x46c) = uVar4;
    }
    uVar8 = uVar8 + 1;
  } while (uVar8 < 0xe);
  bVar11 = 6;
  uVar6 = 0xe;
  do {
    if ((1 << (uVar6 & 0xff) & DAT_0000872c) != 0) {
      bVar1 = *(byte *)(DAT_00008724 + uVar6 + -0x6c);
      if ((1 << (uint)bVar1 & uVar10) != 0) {
        bVar11 = bVar1;
      }
      *(byte *)(param_1 + uVar6 + 0x24) = bVar11;
      iVar7 = uVar6 * 2 + DAT_00008728;
      airtime_compute(0,iVar7 + 0x48,bVar11,uVar2,0xe,1);
      airtime_compute(0,iVar7 + 0x74,bVar11,uVar2,0x20,1);
      uVar4 = pas_rate_phy_class(uVar2,bVar11,1);
      *(undefined1 *)(DAT_00008728 + uVar6 + 0x46c) = uVar4;
    }
    iVar3 = DAT_00008728;
    iVar7 = DAT_00008724;
    uVar6 = uVar6 + 1;
  } while (uVar6 < 0x16);
  uVar6 = 0xe;
  do {
    bVar1 = *(byte *)(iVar7 + uVar6 + -100);
    if ((1 << (uint)bVar1 & uVar10) != 0) {
      bVar11 = bVar1;
    }
    iVar9 = iVar3 + uVar6;
    uVar6 = uVar6 + 1;
    *(byte *)(iVar9 + 0x47a) = bVar11;
  } while (uVar6 < 0x16);
  return;
}



/* ======================================================================
 * 00008676  pas_program_rate_tables
 * ====================================================================== */

void pas_program_rate_tables(int param_1)

{
  byte bVar1;
  undefined2 uVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  uint uVar9;
  
  if (*(char *)(param_1 + 0x21) == '\0') {
    iVar6 = 0;
  }
  else {
    iVar6 = 2;
  }
  pas_build_rate_tables(param_1);
  uVar7 = 0;
  uVar2 = *(undefined2 *)(DAT_00008728 + 2);
  do {
    uVar8 = (uint)*(byte *)(DAT_00008724 + 0x16 + uVar7 * 2);
    uVar9 = (uint)*(byte *)(uVar7 * 2 + DAT_00008724 + 0x16 + 1);
    uVar3 = (uint)*(byte *)(param_1 + uVar9 + 0x24);
    bVar1 = *(byte *)(DAT_00008724 + uVar9);
    iVar4 = uVar8 * 4 + DAT_00008724 + -0x2e;
    uVar5 = (uint)*(byte *)(iVar4 + iVar6) + (uint)bVar1 & 0xff;
    if (uVar8 == 2) {
      uVar5 = uVar5 - 8 & 0xff;
    }
    pas_build_rate_entry
              (uVar2,uVar8,uVar3,*(undefined1 *)(DAT_00008724 + uVar3),
               uVar5 * 0x10 + DAT_00008730 + DAT_00008734);
    if (3 < uVar8) {
      uVar3 = (uint)*(byte *)(DAT_00008728 + uVar9 + 0x47a);
      pas_build_rate_entry
                (uVar2,uVar8,uVar3,*(undefined1 *)(DAT_00008724 + uVar3),
                 ((uint)*(byte *)(iVar4 + 1) + (uint)bVar1 & 0xff) * 0x10 + DAT_00008730 +
                 DAT_00008734);
    }
    uVar7 = uVar7 + 1;
  } while (uVar7 < 0x1f);
  return;
}



/* ======================================================================
 * 00008738  pas_next_higher_rate
 * ====================================================================== */

uint pas_next_higher_rate(undefined4 param_1,uint param_2,undefined1 *param_3)

{
  int iVar1;
  
  do {
    param_2 = param_2 + 1 & 0xff;
    if (0x15 < param_2) {
      if (param_2 != 0x16) {
        return param_2;
      }
      return 0xff;
    }
    iVar1 = pas_rate_retries_for_idx(param_1,param_2);
    *param_3 = (char)iVar1;
  } while (iVar1 == 0);
  return param_2;
}



/* ======================================================================
 * 0000876a  pas_rate_recovery_on_success
 * ====================================================================== */

/* pas_rate_recovery_on_success(tx_ctx, status) -- firmware rate step-UP.
   Called with status == 0 (success).  This is the only place the firmware
   raises the TX rate on its own.
   
     if (status != 0 || tx_ctx[0x0E] == 0xF) return;
     policy = g_fw_ctx + 0xF0 + index*0x14;
     if ((policy->flags & 3) != 2)      return;   /* mode 2 only */
     if (policy->rate_recoveries == 0)  return;   /* +0xF4, must be non-zero */
   
     state = g_fw_ctx + policy->index*4;
     if (++state[0x373] == policy->rate_recoveries) {
         state[0x373] = 0;
         up = pas_next_higher_rate(policy, state[0x371]);   /* scans up, cap 0x15 */
         if (up != 0xFF) { state[0x371] = up; }
         state[0x372] = retries_for(up or current) - 1;
     }
   
   DEAD IN PRACTICE: mainline and both vendor trees set policyFlags = 0x0C
   (mode 0) and never assign rate_recoveries / rateRecoveryCount, which the
   designated initialiser leaves at 0.  Both guards therefore fail and the
   firmware never steps a rate back up.
   
   To enable it a driver must set (flags & 3) == 2 AND rate_recoveries > 0.
   Note mode 2 also changes the fallback path in pas_tx_retry_advance from
   per-frame to per-policy state, so this is not a drop-in change. */

void pas_rate_recovery_on_success(int param_1,int param_2)

{
  char cVar1;
  int iVar2;
  int iVar3;
  byte *pbVar4;
  
  if ((param_2 == 0) && (*(byte *)(param_1 + 0xe) != 0xf)) {
    iVar3 = (uint)*(byte *)(param_1 + 0xe) * 0x14 + DAT_00008a94;
    pbVar4 = (byte *)(iVar3 + 0xf0);
    if (((*(byte *)(iVar3 + 0xf3) & 3) == 2) && (*(char *)(iVar3 + 0xf4) != '\0')) {
      iVar2 = (uint)*pbVar4 * 4 + DAT_00008a94;
      cVar1 = *(char *)(iVar2 + 0x373) + '\x01';
      *(char *)(iVar2 + 0x373) = cVar1;
      if (cVar1 == *(char *)(iVar3 + 0xf4)) {
        *(undefined1 *)(iVar2 + 0x373) = 0;
        iVar3 = pas_next_higher_rate(pbVar4,*(undefined1 *)(iVar2 + 0x371),iVar2 + 0x372);
        if (iVar3 == 0xff) {
          cVar1 = pas_rate_retries_for_idx(pbVar4,*(undefined1 *)(iVar2 + 0x371));
        }
        else {
          *(char *)(iVar2 + 0x371) = (char)iVar3;
          cVar1 = *(char *)(iVar2 + 0x372);
        }
        *(char *)(iVar2 + 0x372) = cVar1 + -1;
      }
    }
  }
  return;
}



/* ======================================================================
 * 000087d2  pas_tx_policy_prepare
 * ====================================================================== */

undefined4 pas_tx_policy_prepare(int param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  byte bVar4;
  
  iVar1 = DAT_00008a94;
  uVar2 = (uint)*(byte *)(param_1 + 0xe);
  iVar3 = uVar2 * 0x14 + DAT_00008a94;
  *(undefined1 *)(param_1 + 0x57) = 0xff;
  bVar4 = *(byte *)(iVar3 + 0xf3) & 3;
  if ((*(byte *)(iVar3 + 0xf3) & 3) != 0) {
    if (bVar4 == 1) {
      *(undefined1 *)(param_1 + 0xf) = *(undefined1 *)(uVar2 * 4 + iVar1 + 0x370);
    }
    else if (bVar4 == 2) {
      *(undefined1 *)(param_1 + 0xf) = *(undefined1 *)(uVar2 * 4 + iVar1 + 0x371);
      *(undefined1 *)(param_1 + 0x57) = 0;
    }
  }
  if (((((*(byte *)(iVar3 + 0xf3) & 3) != 0) && (uVar2 != 0xf)) && (*(int *)(iVar3 + 0xf8) == 0)) &&
     ((*(int *)(iVar3 + 0xfc) == 0 && (*(int *)(iVar3 + 0x100) == 0)))) {
    return 0;
  }
  return 1;
}



/* ======================================================================
 * 00008838  pas_backoff_reset
 * ====================================================================== */

void pas_backoff_reset(int param_1,int param_2)

{
  int *piVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  piVar1 = DAT_00008a9c;
  iVar3 = param_1 * 0x98 + DAT_00008a94;
  iVar4 = iVar3 + param_2 * 4;
  iVar3 = iVar3 + DAT_00008a98;
  *(undefined4 *)(iVar4 + 0x4ac) = 0;
  if (*piVar1 == 0) {
    uVar2 = (uint)*(ushort *)(iVar3 + param_2 * 2);
  }
  else {
    uVar2 = piVar1[1];
  }
  *(uint *)(iVar4 + 0x4bc) = uVar2;
  return;
}



/* ======================================================================
 * 00008868  pas_tx_retry_advance
 * ====================================================================== */

/* pas_tx_retry_advance(tx_ctx, is_probe) -- per-retry rate/backoff decision.
   The core of TX rate fallback.  Calls pas_backoff_cw_update on every retry.
   
   tx_ctx fields used:
     +0x04 flags   bits 9:10 select short vs long frame retry limit,
                   bit 5 gates committing a new rate, 0x80000 = rate changed,
                   0xC0000 = also dropped from OFDM/HT into CCK
     +0x0C AC / queue id          +0x0E rate policy index (0xF = none)
     +0x0F current rate index     +0x1E accumulated try count
     +0x57 retries left (policy mode 0)   +0x69 link id
   
   policy = g_fw_ctx + 0xF0 + index*0x14, its flags byte at +0xF3.
   
   TWO DISTINCT MODES, selected by (policy->flags & 3):
   
     mode 2 : uses the per-policy cached walk at g_fw_ctx + index*4 + 0x370
              [0]=highest rate  [1]=current rate  [2]=retries left
              [3]=consecutive-success counter (see pas_rate_recovery_on_success)
              On exhausting the current rate it calls pas_next_lower_rate; if
              there is none, BIT(2) TERMINATE_WHEN_FINISHED decides give-up vs
              restart at the same rate.
   
     mode 0/3 : no per-policy state at all.  The rate is recomputed per frame
              by pas_rate_for_try_count(policy, tries) -- a pure function of
              the accumulated try count over the nibble table -- and retries
              left are kept in the frame's own tx_ctx+0x57.
   
   Committing a new rate:  if (!(flags & BIT(5)) || rate > 13) tx_ctx[0xF] = rate.
   Rates 14..21 are MCS0..MCS7, so an HT rate is always committed.
   On any rate change it sets 0x80000 (and 0xC0000 when falling below index 4)
   and calls FUN_00007FA6 (pac_phy.c) to reconfigure the PHY.
   
   CONCLUSION FOR THE THROUGHPUT WORK: mainline and both vendor trees set
   policyFlags = BIT(2)|BIT(3) = 0x0C, i.e. mode 0.  In mode 0 the firmware
   keeps NO rate state across frames -- it is a pure executor of the host's
   per-frame retry policy table, with rate selection owned by mac80211
   (minstrel_ht).  There is no firmware-side rate adaptation to blame for the
   monotonic run-to-run degradation. */

int pas_tx_retry_advance(xr_tx_pas *pas,int is_probe)

{
  char cVar1;
  byte bVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  byte *pbVar7;
  byte bVar8;
  bool bVar9;
  char local_30 [4];
  uint local_2c;
  byte *local_28;
  uint local_24;
  undefined1 *local_20;
  xr_tx_pas *pxStack_1c;
  int local_18;
  
  iVar4 = DAT_00008a94;
  local_20 = &pas->field_0x60;
  uVar3 = *(uint *)((uint)pas->bIfId * 0x98 + DAT_00008a94 + (uint)pas->bAc * 4 + 0x4ac);
  pxStack_1c = pas;
  local_18 = is_probe;
  if (pas->bRatePolicyIdx == 0xf) {
    uVar5 = 4;
    if ((pas->dwFlags & 0x7ff) >> 9 == 0) {
      uVar5 = 7;
    }
    bVar9 = uVar5 <= uVar3;
    do {
      if (bVar9) goto LAB_00008a20;
      bVar9 = true;
    } while (uVar5 <= pas->wTryCount);
    uVar3 = *(uint *)(DAT_00008aa0 + 0x3c) & 0x3f;
    if (((*(ushort *)&pas->field_0xa & 0xff) == 0xa4) &&
       ((int)(*(uint *)(DAT_00008aa0 + 0x3c) << 7) < 0)) {
      if (uVar3 == 0xf) {
        return 0;
      }
      if (uVar3 == 0x10) {
        return 0;
      }
      if (uVar3 == 0x13) {
        return 0;
      }
      if (uVar3 == 0x14) {
        return 0;
      }
    }
    pas_backoff_cw_update(pas);
    goto LAB_00008a18;
  }
  iVar6 = (uint)pas->bRatePolicyIdx * 0x14 + DAT_00008a94;
  pbVar7 = (byte *)(iVar6 + 0xf0);
  if ((*(byte *)(iVar6 + 0xf3) & 3) == 2) {
    if ((pas->dwFlags & 0x7ff) >> 9 == 0) {
      bVar8 = *(byte *)(iVar6 + 0xf1);
    }
    else {
      bVar8 = *(byte *)(iVar6 + 0xf2);
    }
    local_2c = (uint)bVar8;
    local_24 = uVar3;
    pas_backoff_cw_update(pas);
    iVar4 = (uint)*pbVar7 * 4 + iVar4;
    if (local_18 != 0) {
      return 1;
    }
    *(undefined1 *)(iVar4 + 0x373) = 0;
    bVar8 = *(byte *)(iVar4 + 0x371);
    if ((local_2c <= local_24) ||
       ((local_2c <= pas->wTryCount && ((pas->dwFlags & 0x7ff) >> 9 != 0)))) {
LAB_00008a20:
      pas_backoff_reset(local_20[9],pas->bAc);
      return 0;
    }
    cVar1 = *(char *)(iVar4 + 0x372);
    if (*(char *)(iVar4 + 0x372) == '\0') {
      uVar3 = pas_next_lower_rate(pbVar7,bVar8,local_30);
      if (uVar3 != 0xff) {
        *(byte *)(iVar4 + 0x371) = (byte)uVar3;
        cVar1 = local_30[0];
        if ((-1 < (int)(pas->dwFlags << 0x1a)) || (0xd < uVar3)) {
          pas->bRateIdx = (byte)uVar3;
        }
        goto LAB_0000893e;
      }
      if ((int)((uint)*(byte *)(iVar6 + 0xf3) << 0x1d) < 0) goto LAB_00008a20;
      cVar1 = pas_rate_retries_for_idx(pbVar7,*(undefined1 *)(iVar4 + 0x371));
    }
    else {
LAB_0000893e:
      cVar1 = cVar1 + -1;
    }
    *(char *)(iVar4 + 0x372) = cVar1;
  }
  else {
    pas_backoff_cw_update(pas);
    local_28 = &pas->bRetriesLeft;
    uVar5 = pas->dwFlags;
    bVar8 = pas->bRateIdx;
    if ((uVar5 & 0x7ff) >> 9 == 0) {
      bVar2 = *(byte *)(iVar6 + 0xf1);
    }
    else {
      bVar2 = *(byte *)(iVar6 + 0xf2);
    }
    if ((((bVar2 <= uVar3) && (local_18 != 0)) && ((int)(uVar5 << 0x1a) < 0)) ||
       ((uint)bVar2 <= (uint)pas->wTryCount)) goto LAB_00008a20;
    if (local_18 != 0) {
      pas->dwFlags = uVar5 & 0xfff7ffff;
      if ((uVar3 & 1) == 0) {
        return 1;
      }
      goto LAB_00008a18;
    }
    uVar3 = pas_rate_for_try_count(pbVar7,pas->wTryCount & 0xff,&local_2c);
    if (uVar3 == 0xff) {
      if ((int)((uint)*(byte *)(iVar6 + 0xf3) << 0x1d) < 0) goto LAB_00008a20;
      bVar2 = pas_rate_retries_for_idx(pbVar7,pas->bRateIdx);
    }
    else {
      if ((-1 < (int)(pas->dwFlags << 0x1a)) || (0xd < uVar3)) {
        pas->bRateIdx = (byte)uVar3;
      }
      bVar2 = (char)local_2c - 1;
    }
    *local_28 = bVar2;
  }
  if (pas->bRateIdx != bVar8) {
    uVar3 = pas->dwFlags;
    pas->dwFlags = uVar3 | 0x80000;
    if ((3 < bVar8) && (pas->bRateIdx < 4)) {
      pas->dwFlags = uVar3 | 0xc0000;
    }
    pas_compute_tx_timing(pas);
  }
LAB_00008a18:
  tx_bump_try_count(pas);
  return 1;
}



/* ======================================================================
 * 00008a2c  pas_tx_retry_advance_mode0
 * ====================================================================== */

undefined4
pas_tx_retry_advance_mode0
          (xr_tx_pas *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  byte bVar1;
  int iVar2;
  uint uVar3;
  undefined4 uStack_10;
  
  if (-1 < (int)(param_1->dwFlags << 3)) {
    iVar2 = (uint)param_1->bRatePolicyIdx * 0x14 + DAT_00008a94;
    if ((param_1->dwFlags & 0x7ff) >> 9 == 0) {
      bVar1 = *(byte *)(iVar2 + 0xf1);
    }
    else {
      bVar1 = *(byte *)(iVar2 + 0xf2);
    }
    if (((ushort)bVar1 <= param_1->wTryCount) || (param_1->bRatePolicyIdx == 0xf)) {
      return 0;
    }
    if ((*(byte *)(iVar2 + 0xf3) & 3) == 0) {
      uStack_10 = param_4;
      uVar3 = pas_rate_for_try_count(iVar2 + 0xf0,param_1->wTryCount & 0xff,&uStack_10);
      if (((param_1->bRateIdx != uVar3) && (uVar3 != 0xff)) &&
         ((-1 < (int)(param_1->dwFlags << 0x1a) || (0xd < uVar3)))) {
        param_1->bRateIdx = (byte)uVar3;
        pas_compute_tx_timing(param_1);
      }
      tx_bump_try_count(param_1);
    }
  }
  return 1;
}



/* ======================================================================
 * 00008aa4  rxfifo_slot_valid
 * ====================================================================== */

undefined4 rxfifo_slot_valid(uint param_1)

{
  if (0x6fff < param_1) {
    param_1 = param_1 - 0x7000;
  }
  if (*(int *)(DAT_00008df8 + param_1) == DAT_00008dfc) {
    return 1;
  }
  return 0;
}



/* ======================================================================
 * 00008ac2  rxfifo_wrap_sub
 * ====================================================================== */

uint rxfifo_wrap_sub(uint param_1,uint param_2)

{
  param_1 = param_1 & DAT_00008e00;
  if (param_2 <= param_1) {
    param_1 = param_1 - param_2;
  }
  return param_1;
}



/* ======================================================================
 * 00008ace  rxfifo_advance
 * ====================================================================== */

uint rxfifo_advance(int param_1,int param_2)

{
  uint uVar1;
  
  uVar1 = param_1 + param_2 + 0x2fU & DAT_00008e00;
  if (0x6fff < uVar1) {
    uVar1 = uVar1 - 0x7000;
  }
  return uVar1;
}



/* ======================================================================
 * 00008ae2  rxfifo_off_to_addr
 * ====================================================================== */

int rxfifo_off_to_addr(uint param_1)

{
  if (0x6fff < param_1) {
    param_1 = param_1 - 0x7000;
  }
  return DAT_00008df8 + param_1;
}



/* ======================================================================
 * 00008af2  rxfifo_next_frame
 * ====================================================================== */

int rxfifo_next_frame(void)

{
  int iVar1;
  undefined4 uVar2;
  uint uVar3;
  undefined4 uVar4;
  uint uVar5;
  int iVar6;
  undefined4 *puVar7;
  int iVar8;
  int iVar9;
  uint uVar10;
  undefined4 uVar11;
  
  iVar9 = 0;
  uVar2 = irq_disable_save();
  iVar1 = DAT_00008e08;
  uVar10 = *(uint *)(DAT_00008e04 + 4);
  uVar3 = *(uint *)(DAT_00008e08 + 0x14);
  if (uVar3 != uVar10) {
    uVar11 = *(undefined4 *)(DAT_00008e04 + 4);
    if (uVar10 < uVar3) {
      uVar3 = (uVar10 - uVar3) + 0x7000;
    }
    else {
      uVar3 = uVar10 - uVar3;
    }
    iVar9 = rxfifo_off_to_addr();
    uVar4 = rxfifo_advance(*(undefined4 *)(iVar1 + 0x14),*(undefined2 *)(iVar9 + 0x18));
    uVar5 = rxfifo_wrap_sub(uVar4,0x7000);
    iVar6 = rxfifo_slot_valid();
    if ((iVar6 == 0) || (uVar3 < *(ushort *)(iVar9 + 0x18))) {
      iVar6 = rxfifo_slot_valid(uVar5);
      if ((iVar6 == 0) && ((*(ushort *)(iVar9 + 0x18) <= uVar3 && (uVar10 == uVar5)))) {
        *(uint *)(iVar1 + 0x14) = uVar5;
        *(uint *)(iVar9 + 8) = *(uint *)(iVar9 + 8) & 0xff;
        *(uint *)(iVar9 + 4) = uVar5;
        puVar7 = (undefined4 *)rxfifo_off_to_addr(*(undefined4 *)(iVar1 + 0x14));
        *puVar7 = DAT_00008dfc;
      }
      else {
        do {
          do {
            iVar6 = DAT_00008e04;
            if (uVar3 == 0) {
              *(undefined4 *)(iVar1 + 0x10) = uVar11;
              *(undefined4 *)(iVar1 + 0x14) = uVar11;
              iVar9 = 0;
              *(undefined4 *)(iVar6 + 8) = uVar11;
              goto LAB_00008bda;
            }
            uVar3 = uVar3 - 4;
            *(int *)(iVar1 + 0x14) = *(int *)(iVar1 + 0x14) + 4;
            iVar9 = rxfifo_slot_valid();
          } while (iVar9 == 0);
          iVar9 = rxfifo_off_to_addr(*(undefined4 *)(iVar1 + 0x14));
          uVar4 = rxfifo_advance(*(undefined4 *)(iVar1 + 0x14),*(undefined2 *)(iVar9 + 0x18));
          uVar4 = rxfifo_wrap_sub(uVar4,0x7000);
        } while ((uVar3 < *(ushort *)(iVar9 + 0x18)) ||
                (iVar8 = rxfifo_slot_valid(uVar4), iVar6 = DAT_00008e04, iVar8 == 0));
        *(undefined4 *)(iVar1 + 0x10) = *(undefined4 *)(iVar1 + 0x14);
        *(undefined4 *)(iVar1 + 0x14) = uVar4;
        *(undefined4 *)(iVar9 + 4) = uVar4;
        *(uint *)(iVar9 + 8) = *(uint *)(iVar9 + 8) & 0xff;
        *(undefined4 *)(iVar6 + 8) = *(undefined4 *)(iVar1 + 0x10);
      }
    }
    else {
      *(uint *)(iVar1 + 0x14) = uVar5;
      *(uint *)(iVar9 + 4) = uVar5;
      *(uint *)(iVar9 + 8) = *(uint *)(iVar9 + 8) & 0xff;
    }
  }
LAB_00008bda:
  irq_restore(uVar2);
  if ((iVar9 != 0) && (*(uint *)(iVar9 + 8) >> 8 != 0)) {
    fw_assert(s_rx_fifo_c_00008e0c,0xcf,0x12);
  }
  if ((*(int *)(iVar1 + 0x10) - *(int *)(iVar1 + 0x14)) + 0x7000U < 0x1000) {
    *(undefined4 *)(iVar1 + 0x18) = *DAT_00008e18;
  }
  return iVar9;
}



/* ======================================================================
 * 00008c12  rxfifo_addr_to_off
 * ====================================================================== */

uint rxfifo_addr_to_off(int param_1)

{
  uint uVar1;
  
  uVar1 = param_1 - DAT_00008df8;
  if (0x6fff < uVar1) {
    uVar1 = uVar1 - 0x7000;
  }
  return uVar1;
}



/* ======================================================================
 * 00008c22  rxfifo_release_slot
 * ====================================================================== */

void rxfifo_release_slot(undefined4 *param_1)

{
  int iVar1;
  undefined4 uVar2;
  int iVar3;
  undefined4 uVar4;
  undefined4 *puVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  
  uVar2 = irq_disable_save();
  if ((param_1 < DAT_00008df8) || (DAT_00008e1c < param_1)) {
    fw_assert(s_rx_fifo_c_00008e0c,0x139,0x13);
  }
  iVar1 = DAT_00008e08;
  uVar8 = param_1[2] & 0xff;
  uVar7 = param_1[2] & 0xffffff00;
  if (uVar7 != DAT_00008e20) {
    if (uVar7 != 0xffffff00) {
      if (uVar7 == 0) {
        param_1[2] = uVar8 - 0x100;
      }
      else {
        fw_assert(s_rx_fifo_c_00008e0c,0x153,0x14);
      }
    }
    iVar3 = rxfifo_addr_to_off(param_1);
    if (*(int *)(iVar1 + 0x10) == iVar3) {
      iVar3 = rxfifo_slot_valid(param_1[1]);
      if (iVar3 == 0) {
        fw_assert(s_rx_fifo_c_00008e0c,0x161,0x15);
      }
      uVar4 = rxfifo_wrap_sub(param_1[1],0x7000);
      *(undefined4 *)(iVar1 + 0x10) = uVar4;
      iVar3 = DAT_00008e04;
      param_1[2] = uVar8 + DAT_00008e20;
      *param_1 = 0;
      uVar4 = *(undefined4 *)(iVar1 + 0x10);
      do {
        *(undefined4 *)(iVar3 + 8) = uVar4;
        while( true ) {
          if (*(int *)(iVar1 + 0x10) == *(int *)(iVar1 + 0x14)) goto LAB_00008cfc;
          puVar5 = (undefined4 *)rxfifo_off_to_addr();
          iVar6 = rxfifo_slot_valid(puVar5[1]);
          if (iVar6 == 0) {
            fw_assert(s_rx_fifo_c_00008e0c,0x183,0x16);
          }
          uVar7 = puVar5[2];
          uVar8 = uVar7 & 0xffffff00;
          if (uVar8 == 0xffffff00) break;
          if (uVar8 == 0) goto LAB_00008cfc;
          fw_assert(s_rx_fifo_c_00008e0c,0x1a7,0x18);
        }
        iVar6 = rxfifo_slot_valid(puVar5[1]);
        if (iVar6 == 0) {
          fw_assert(s_rx_fifo_c_00008e0c,0x196,0x17);
        }
        uVar4 = rxfifo_wrap_sub(puVar5[1],0x7000);
        *(undefined4 *)(iVar1 + 0x10) = uVar4;
        puVar5[2] = (uVar7 & 0xff) + DAT_00008e20;
        *puVar5 = 0;
        uVar4 = *(undefined4 *)(iVar1 + 0x10);
      } while( true );
    }
  }
LAB_00008cfc:
  irq_restore(uVar2);
  if ((*(int *)(iVar1 + 0x18) != 0) &&
     (0xfff < (*(int *)(iVar1 + 0x10) - *(int *)(iVar1 + 0x14)) + 0x7000U)) {
    *DAT_00008e18 = *(int *)(iVar1 + 0x18);
    *(undefined4 *)(iVar1 + 0x18) = 0;
  }
  if (*(int *)(iVar1 + 0x10) == *(int *)(iVar1 + 0x14)) {
    evt_flags_clear(0x80);
  }
  return;
}



/* ======================================================================
 * 00008d66  rxfifo_release_slot
 * ====================================================================== */

void rxfifo_release_slot(undefined4 *param_1)

{
  int iVar1;
  undefined4 uVar2;
  int iVar3;
  undefined4 uVar4;
  undefined4 *puVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  
  uVar2 = irq_disable_save();
  if ((param_1 < DAT_00008df8) || (DAT_00008e1c < param_1)) {
    fw_assert(s_rx_fifo_c_00008e0c,0x139,0x13);
  }
  iVar1 = DAT_00008e08;
  uVar8 = param_1[2] & 0xff;
  uVar7 = param_1[2] & 0xffffff00;
  if (uVar7 != DAT_00008e20) {
    if (uVar7 != 0xffffff00) {
      if (uVar7 == 0) {
        param_1[2] = uVar8 - 0x100;
      }
      else {
        fw_assert(s_rx_fifo_c_00008e0c,0x153,0x14);
      }
    }
    iVar3 = rxfifo_addr_to_off(param_1);
    if (*(int *)(iVar1 + 0x10) == iVar3) {
      iVar3 = rxfifo_slot_valid(param_1[1]);
      if (iVar3 == 0) {
        fw_assert(s_rx_fifo_c_00008e0c,0x161,0x15);
      }
      uVar4 = rxfifo_wrap_sub(param_1[1],0x7000);
      *(undefined4 *)(iVar1 + 0x10) = uVar4;
      iVar3 = DAT_00008e04;
      param_1[2] = uVar8 + DAT_00008e20;
      *param_1 = 0;
      uVar4 = *(undefined4 *)(iVar1 + 0x10);
      do {
        *(undefined4 *)(iVar3 + 8) = uVar4;
        while( true ) {
          if (*(int *)(iVar1 + 0x10) == *(int *)(iVar1 + 0x14)) goto LAB_00008cfc;
          puVar5 = (undefined4 *)rxfifo_off_to_addr();
          iVar6 = rxfifo_slot_valid(puVar5[1]);
          if (iVar6 == 0) {
            fw_assert(s_rx_fifo_c_00008e0c,0x183,0x16);
          }
          uVar7 = puVar5[2];
          uVar8 = uVar7 & 0xffffff00;
          if (uVar8 == 0xffffff00) break;
          if (uVar8 == 0) goto LAB_00008cfc;
          fw_assert(s_rx_fifo_c_00008e0c,0x1a7,0x18);
        }
        iVar6 = rxfifo_slot_valid(puVar5[1]);
        if (iVar6 == 0) {
          fw_assert(s_rx_fifo_c_00008e0c,0x196,0x17);
        }
        uVar4 = rxfifo_wrap_sub(puVar5[1],0x7000);
        *(undefined4 *)(iVar1 + 0x10) = uVar4;
        puVar5[2] = (uVar7 & 0xff) + DAT_00008e20;
        *puVar5 = 0;
        uVar4 = *(undefined4 *)(iVar1 + 0x10);
      } while( true );
    }
  }
LAB_00008cfc:
  irq_restore(uVar2);
  if ((*(int *)(iVar1 + 0x18) != 0) &&
     (0xfff < (*(int *)(iVar1 + 0x10) - *(int *)(iVar1 + 0x14)) + 0x7000U)) {
    *DAT_00008e18 = *(int *)(iVar1 + 0x18);
    *(undefined4 *)(iVar1 + 0x18) = 0;
  }
  if (*(int *)(iVar1 + 0x10) == *(int *)(iVar1 + 0x14)) {
    evt_flags_clear(0x80);
  }
  return;
}



/* ======================================================================
 * 00008d68  rx_buf_release
 * ====================================================================== */

void rx_buf_release(int param_1,int param_2)

{
  if (param_2 == 0) {
    rxfifo_release_slot(param_1 + -0x20);
    return;
  }
  enc_ctx_free(param_1 + -0x120);
  return;
}



/* ======================================================================
 * 00008d80  rxfifo_find_frame_by_subtype
 * ====================================================================== */

ushort * rxfifo_find_frame_by_subtype(uint param_1)

{
  int iVar1;
  ushort *puVar2;
  undefined4 uVar3;
  int iVar4;
  uint uVar5;
  int *piVar6;
  int iVar7;
  uint uVar8;
  
  uVar8 = *(uint *)(DAT_00008e24 + 0x18);
  piVar6 = (int *)(DAT_00008e08 + 0x40);
  if (param_1 == 0x80) {
    piVar6 = DAT_00008e28;
  }
  iVar7 = *(int *)(DAT_00008e04 + 4);
  do {
    if (*piVar6 == iVar7) {
      return (ushort *)0x0;
    }
    iVar1 = rxfifo_off_to_addr();
    puVar2 = (ushort *)(iVar1 + 0x20);
    if ((*puVar2 & 0xff) == param_1) {
      if (param_1 == 0x94) {
        return puVar2;
      }
      if ((param_1 == 0x80) &&
         (uVar5 = (uint)*(ushort *)(iVar1 + 0x18) + iVar1 + 0x23 & 0xfffffffc,
         *(uint *)(uVar5 + 8) >> 0x1e == 2)) {
        *(uint *)(uVar5 + 8) = *(uint *)(uVar5 + 8) & 0xdfffffff | uVar8 & 0x20000000;
        return puVar2;
      }
    }
    uVar3 = rxfifo_advance(*piVar6,*(undefined2 *)(iVar1 + 0x18));
    iVar1 = rxfifo_wrap_sub(uVar3,0x7000);
    iVar4 = rxfifo_slot_valid();
    if (iVar4 == 0) {
      return (ushort *)0x0;
    }
    *piVar6 = iVar1;
  } while( true );
}



/* ======================================================================
 * 00008e2c  rx_handler_main_loop
 * ====================================================================== */

void rx_handler_main_loop(void)

{
  int iVar1;
  int *piVar2;
  ushort *puVar3;
  ushort *puVar4;
  uint uVar5;
  char *pcVar6;
  ushort *local_40;
  ushort local_3c;
  undefined1 local_3a;
  byte local_39;
  int local_38;
  ushort local_30;
  ushort local_2e;
  undefined1 local_2c;
  char local_2b;
  byte local_2a;
  ushort local_26;
  undefined2 local_24;
  uint local_20;
  int *local_1c;
  
  iVar1 = DAT_0000900c;
  pcVar6 = (char *)(DAT_0000900c + 0x460);
LAB_00008e3e:
  do {
    if (*DAT_00009010 != 0) {
      return;
    }
    piVar2 = (int *)rxfifo_next_frame();
    *(undefined4 *)(DAT_0000900c + 0x18) = 0;
    if (piVar2 == (int *)0x0) {
      *DAT_00009014 = *DAT_00009014;
      return;
    }
    local_1c = piVar2;
    if (*piVar2 != DAT_00009018) {
      fw_assert(s_rx_handler_c_0000901c,0xa0,0x27);
    }
    evt_flags_set(DAT_0000902c,0x80);
    local_38 = piVar2[5];
    local_3c = (short)piVar2[6] - 4;
    local_40 = (ushort *)(piVar2 + 8);
    uVar5 = (int)local_40 + *(ushort *)(piVar2 + 6) + 3 & 0xfffffffc;
    local_30 = *local_40;
    local_2e = local_30 & 0xff;
    local_2b = *(char *)(uVar5 + 7);
    local_2c = *(undefined1 *)(uVar5 + 4);
    local_26 = *(ushort *)(uVar5 + 2);
    local_3a = *(undefined1 *)((int)piVar2 + 0x1a);
    local_39 = (byte)(piVar2[7] & 7U);
    local_2a = *(byte *)((uint)*(byte *)((int)piVar2 + 0x1a) +
                        ((piVar2[7] & 7U) >> 1) * 0x10 + DAT_00009030);
    local_24 = *(undefined2 *)(DAT_00009034 + 0x10);
    local_20 = 0;
    trace_push_pair(local_30,local_3c | 0x8000);
    if (local_39 < 3) {
      local_26 = local_26 & 0x3ff;
    }
    else {
      uVar5 = local_20 | 0x4000;
      if ((piVar2[7] & 0xffU) >> 6 != 0) {
        uVar5 = local_20 | 0xc000;
      }
      local_20 = uVar5;
      if (local_2a < 0xe) {
LAB_00008f1e:
        if (-1 < piVar2[7] << 0x1a) {
          *pcVar6 = '\0';
          goto LAB_00008f72;
        }
      }
      else if (-1 < piVar2[7] << 0x1a) {
        *(char *)(iVar1 + 0x690) = local_2b;
        *(undefined1 *)(iVar1 + 0x691) = local_2c;
        goto LAB_00008f1e;
      }
      if (local_2b == '\0') {
        local_2b = *(char *)(iVar1 + 0x690);
        local_2c = *(undefined1 *)(iVar1 + 0x691);
        if (*pcVar6 == '\0') {
          local_20 = local_20 | 0x10;
        }
        *pcVar6 = '\x01';
      }
      else {
        *(char *)(iVar1 + 0x690) = local_2b;
        *(undefined1 *)(iVar1 + 0x691) = local_2c;
        local_20 = local_20 | 0x20;
        *pcVar6 = '\0';
      }
      local_20 = local_20 | 8;
    }
LAB_00008f72:
    if (((local_30 & 3) == 0) && (9 < local_3c)) {
      if (local_2e == 0x50) {
LAB_00008fe2:
        tsf_sync_from_beacon(&local_40);
      }
      else {
        if (local_2e == 0x80) {
          if ((local_40[0x11] & 0xf) >> 2 != 0) {
            puVar3 = (ushort *)((int)local_40 + (uint)local_3c);
            for (puVar4 = local_40 + 0x12; puVar4 < puVar3;
                puVar4 = (ushort *)((int)puVar4 + *(byte *)((int)puVar4 + 1) + 2)) {
              if ((char)*puVar4 == '\x04') {
                if ((ushort *)((int)puVar4 + *(byte *)((int)puVar4 + 1) + 2) <= puVar3)
                goto LAB_00008fcc;
                break;
              }
            }
            puVar4 = (ushort *)0x0;
LAB_00008fcc:
            if (puVar4 != (ushort *)0x0) {
              mac_extend_txop_if_room
                        ((uint)(byte)puVar4[3] + (uint)*(byte *)((int)puVar4 + 7) * 0x100,local_38);
            }
          }
          goto LAB_00008fe2;
        }
        if ((local_2e == 0xe4) || (local_2e == 0xf4)) {
          mac_set_txop_limit(0);
        }
      }
      rx_mgmt_frame_handler(&local_40);
      goto LAB_00008e3e;
    }
    DAT_00009010[1] = DAT_00009010[1] + 1;
    rxfifo_release_slot(piVar2);
  } while( true );
}



/* ======================================================================
 * 00009000  rx_buf_free
 * ====================================================================== */

void rx_buf_free(int param_1)

{
  rxfifo_release_slot(param_1 + -0x20);
  return;
}



/* ======================================================================
 * 00009038  mac_program_beacon_timer
 * ====================================================================== */

void mac_program_beacon_timer(void)

{
  *(uint *)(DAT_00009058 + 0x14) =
       *(int *)(DAT_00009050 + 0x2c) + (uint)*(ushort *)(DAT_00009054 + 0x12) * 0x400 | 0x80000000;
  return;
}



/* ======================================================================
 * 0000905c  tx_pipes_all_idle
 * ====================================================================== */

undefined4 tx_pipes_all_idle(void)

{
  uint uVar1;
  uint uVar2;
  
  uVar2 = 0;
  uVar1 = 0;
  do {
    if (*(char *)(uVar1 * 0x6c + DAT_00009454 + 0xa3) == '\0') {
      uVar2 = uVar2 | 1 << uVar1 & 0xffU;
    }
    uVar1 = uVar1 + 1 & 0xff;
  } while (uVar1 < 4);
  if (uVar2 == 0xf) {
    return 1;
  }
  return 0;
}



/* ======================================================================
 * 00009094  mac_hw_idle
 * ====================================================================== */

undefined4 mac_hw_idle(void)

{
  if ((((*(uint *)(DAT_00009458 + 0x28) & 0x1fff) >> 8 != 0x12) &&
      ((*(uint *)(DAT_0000945c + 0x10) & 0xfff) >> 4 == 0)) &&
     ((*(uint *)(DAT_0000945c + 0x20) & 0x1fffffff) >> 0x18 == 0)) {
    return 1;
  }
  return 0;
}



/* ======================================================================
 * 000090ba  desc_or_flags
 * ====================================================================== */

void desc_or_flags(int param_1,int param_2)

{
  *(uint *)(param_1 + 4) = param_2 + 0x80U | *(uint *)(param_1 + 4);
  return;
}



/* ======================================================================
 * 000090c4  tx_build_duration_desc
 * ====================================================================== */

void tx_build_duration_desc(undefined4 *param_1,int param_2,uint param_3)

{
  byte bVar1;
  uint *puVar2;
  uint uVar3;
  undefined4 uVar4;
  
  *param_1 = 0;
  uVar3 = fw_rand_masked(*(undefined4 *)
                          ((uint)*(byte *)(param_2 + 0x69) * 0x98 + DAT_00009460 +
                           (uint)*(byte *)(param_2 + 0xc) * 4 + 0x4bc));
  puVar2 = DAT_00009464;
  if (*DAT_00009464 < uVar3) {
    *DAT_00009464 = uVar3;
  }
  if (uVar3 < 8) {
    puVar2[1] = puVar2[1] + 1;
  }
  else if (uVar3 < 0x10) {
    puVar2[2] = puVar2[2] + 1;
  }
  else if (uVar3 < 0x20) {
    puVar2[3] = puVar2[3] + 1;
  }
  else if (uVar3 < 0x40) {
    puVar2[4] = puVar2[4] + 1;
  }
  else if (uVar3 < 0x80) {
    puVar2[5] = puVar2[5] + 1;
  }
  else if (uVar3 < 0x100) {
    puVar2[6] = puVar2[6] + 1;
  }
  else if (uVar3 < 0x200) {
    puVar2[7] = puVar2[7] + 1;
  }
  else {
    puVar2[8] = puVar2[8] + 1;
  }
  *(short *)(param_2 + 0x5a) = (short)uVar3;
  param_1[1] = (uVar3 & 0xfff) << 10;
  if ((int)(param_3 << 0x1e) < 0) {
    uVar4 = 0xc0000000;
  }
  else {
    uVar4 = 0xd8000000;
  }
  param_1[2] = uVar4;
  if ((param_3 & 1) == 0) {
    uVar3 = param_1[2] | 0x4000000;
  }
  else {
    if ((int)(param_3 << 0x1d) < 0) {
      bVar1 = *(byte *)(param_2 + 0x58);
    }
    else {
      bVar1 = *(byte *)(param_2 + 0xf);
    }
    uVar3 = *(int *)(DAT_00009454 + 0x20) +
            *(int *)(DAT_00009454 + 0x1c) * 2 +
            (uint)*(ushort *)(DAT_0000946c + (uint)*(byte *)((uint)bVar1 + DAT_00009468) * 2) &
            0xffff;
    if ((((int)(param_3 << 0x1d) < 0) &&
        ((*(uint *)((uint)*(byte *)(param_2 + 0x69) * 0x98 + DAT_00009460 + 0x500) & 1) != 0)) &&
       ((DAT_00009470 & ~*(uint *)(param_2 + 4)) == 0)) {
      uVar3 = *(ushort *)(param_2 + 0x34) + uVar3 & 0xffff;
    }
    uVar3 = (uVar3 & 0x3ff) * 8 + 0x2000 | param_1[2];
  }
  param_1[2] = uVar3;
  return;
}



/* ======================================================================
 * 000091e2  desc_freelist_push
 * ====================================================================== */

void desc_freelist_push(undefined4 *param_1)

{
  undefined4 *puVar1;
  
  puVar1 = DAT_00009474;
  *param_1 = *DAT_00009474;
  *puVar1 = param_1;
  return;
}



/* ======================================================================
 * 000091ec  txp_fn_2441
 * ====================================================================== */

void txp_fn_2441(int param_1,char *param_2,int param_3)

{
  ushort uVar1;
  undefined4 uVar2;
  int iVar3;
  uint uVar4;
  int iVar5;
  uint uVar6;
  int iVar7;
  uint uVar8;
  int local_1c;
  
  uVar2 = fw_read_timer();
  local_1c = 10;
  uVar8 = (uint)*(byte *)(param_1 + 0x6c);
  if (*param_2 != '\0') {
    if (*param_2 != '\x01') goto LAB_00009394;
    if (uVar8 < 8) {
      link_set_state(uVar8,10);
    }
    desc_freelist_push(*(undefined4 *)(param_2 + 0x10));
  }
  pas_rate_recovery_on_success(param_1,param_3);
  param_2[0xc] = '\0';
  param_2[0xd] = '\0';
  param_2[0xe] = '\0';
  param_2[0xf] = '\0';
  do {
    if (*param_2 == '\x01') {
      *(uint *)(param_1 + 0x2c) = *(uint *)(param_1 + 0x2c) | 0x400;
      *(ushort *)(param_1 + 0x50) = *(ushort *)(param_1 + 0x50) | 1;
    }
    *(ushort *)(param_1 + 0x50) =
         (ushort)((uint)*(undefined4 *)(param_1 + 4) >> 0x12) & 0xc | *(ushort *)(param_1 + 0x50);
    *(short *)(param_1 + 0x1c) = (short)param_3;
    *(undefined4 *)(param_1 + 0x14) = uVar2;
    if (*param_2 == '\x01') {
      if (param_3 == 0) {
        evt_flags_set(DAT_00009478,0x200000);
      }
      else if (param_3 == 0xb) {
        local_1c = 0xb;
      }
    }
    else {
      *(uint *)(param_1 + 0x2c) = *(uint *)(param_1 + 0x2c) | 0x800;
      tx_ctx_free_inner(param_1);
    }
    param_1 = *(int *)(param_1 + 0x3c);
  } while (param_1 != 0);
  if (*param_2 != '\x01') goto LAB_00009394;
  if (uVar8 < 8) {
    link_set_state(uVar8,local_1c);
  }
  iVar5 = DAT_0000947c;
  if (local_1c != 0xb) {
    iVar3 = rxfifo_find_frame_by_subtype(0x94);
    if (iVar3 == 0) {
      if ((*(char *)(*(int *)(DAT_00009474 + -0xc) + 2) !=
           *(char *)(*(int *)(DAT_00009474 + -0xc) + 1)) || (7 < uVar8)) goto LAB_00009394;
      iVar5 = uVar8 * 0x38 + iVar5;
      *(undefined4 *)(iVar5 + 0x18) = 0;
      *(undefined4 *)(iVar5 + 0x1c) = 0;
    }
    else {
      uVar6 = 8;
      uVar4 = 0;
      do {
        iVar7 = uVar4 * 0x98 + DAT_00009460;
        if (((*(short *)(iVar3 + 4) == *(short *)(iVar7 + 0x47c)) &&
            (*(short *)(iVar3 + 6) == *(short *)(iVar7 + 0x47e))) &&
           (*(short *)(iVar3 + 8) == *(short *)(iVar7 + 0x480))) {
          if (*(int *)(uVar4 * 0x3b0 + DAT_00009480 + 0x1c) << 0x1d < 0) {
            iVar7 = iVar3 + 10;
          }
          else {
            iVar7 = 0;
          }
          uVar6 = txpipe_find_by_mac_tid(uVar4,*(ushort *)(iVar3 + 0x10) >> 0xc,iVar7);
          break;
        }
        uVar4 = uVar4 + 1 & 0xff;
      } while (uVar4 < 2);
      *(int *)(DAT_00009484 + 0x1c) = *(int *)(DAT_00009484 + 0x1c) + 1;
      if ((uVar6 < 8) && (uVar6 == uVar8)) {
        iVar5 = uVar6 * 0x38 + iVar5;
        if (4 < *(byte *)(iVar5 + 0x10)) {
          uVar1 = *(ushort *)(iVar3 + 0x12);
          uVar2 = *(undefined4 *)(iVar3 + 0x18);
          *(undefined4 *)(iVar5 + 0x18) = *(undefined4 *)(iVar3 + 0x14);
          *(undefined4 *)(iVar5 + 0x1c) = uVar2;
          *(ushort *)(iVar5 + 0x16) = uVar1 >> 4;
          bab_process_ba_bitmap(uVar6);
        }
        goto LAB_00009394;
      }
      if (7 < uVar8) {
        fw_assert(s_tx_ptcs_c_0000948c,DAT_00009488,0x37);
      }
    }
    link_set_state(uVar8,0xb);
  }
  evt_flags_set(DAT_00009478,0x200000);
LAB_00009394:
  *(char *)(DAT_00009498 + 0x14) = *(char *)(DAT_00009498 + 0x14) + -1;
  return;
}



/* ======================================================================
 * 0000939e  txp_program_pipe_hw
 * ====================================================================== */

undefined4 txp_program_pipe_hw(undefined4 *param_1,undefined4 *param_2)

{
  ushort uVar1;
  bool bVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  int iVar8;
  int iVar9;
  ushort uVar10;
  
  uVar3 = (uint)*(byte *)((int)param_1 + 0x69);
  if (2 < uVar3) {
    return 1;
  }
  iVar6 = uVar3 * 0x98 + DAT_00009460;
  bVar2 = false;
  iVar9 = uVar3 * 0x3b0 + DAT_00009480;
  uVar3 = *(uint *)(iVar9 + 0x1c);
  if (*(char *)(iVar6 + 0x492) == '\0') {
    if (*(char *)(iVar6 + 0x493) == '\0') {
      if (((int)(uVar3 << 2) < 0) && ((~uVar3 & 3) == 0)) goto LAB_00009400;
    }
    else {
      iVar4 = tsf_read_low();
      iVar4 = *(int *)(iVar6 + 0x474) - iVar4;
      if ((iVar4 < 1) ||
         (iVar4 <= (int)((uint)*(ushort *)(param_1 + 0xc) + (uint)*(ushort *)(param_1 + 0xd) +
                        (uint)*(ushort *)(param_1 + 0xe) + (uint)*(ushort *)((int)param_1 + 0x36))))
      goto LAB_00009400;
    }
  }
  else {
LAB_00009400:
    bVar2 = true;
  }
  if (((*(char *)((int)param_1 + 0xe) != '\x0f') || (-1 < *DAT_0000949c << 0x18)) && (bVar2)) {
    if (param_2 != (undefined4 *)0x0) {
      *param_2 = 1;
    }
    return 0;
  }
  if (-1 < (int)(uVar3 << 0x1d)) {
    return 1;
  }
  uVar7 = 1 << *(sbyte *)((int)param_1 + 0x6b);
  uVar5 = uVar7 & 0xffff;
  uVar1 = *(ushort *)(iVar9 + 0x15c);
  if ((((((uVar1 & uVar5) == 0) || (uVar10 = *(ushort *)((int)param_1 + 10) & 0xff, uVar10 == 0x50))
       || ((uVar10 == 0xd0 && (*(char *)((int)param_1 + 0xe) == '\x0f')))) &&
      ((((uVar7 & 1) == 0 || (uVar1 == 0)) || (*(char *)(iVar9 + 0x164) == '\0')))) &&
     (-1 < (int)(uVar3 << 2))) {
    return 1;
  }
  uVar10 = (ushort)uVar7;
  if ((*(ushort *)(iVar9 + 0x15e) & uVar5) == 0) {
    if ((*(ushort *)(iVar9 + 0x160) & uVar5) == 0) {
      if (param_2 != (undefined4 *)0x0) {
        *param_2 = 0;
      }
      iVar6 = DAT_000098b8;
      if ((*(ushort *)(DAT_000098b8 + 0x16) & uVar5) != 0) {
        return 0;
      }
      if ((*(ushort *)(iVar9 + 0x2c) & uVar5) == 0) {
        return 0;
      }
      iVar4 = DAT_000098b8 - DAT_000098bc;
      for (iVar9 = 0; iVar9 < (int)(uint)*(ushort *)(iVar6 + 0x14); iVar9 = iVar9 + 1) {
        iVar8 = iVar9 * 0xc + iVar4 + DAT_000098bc;
        if (*(char *)((int)param_1 + 0x6b) == *(char *)(iVar8 + 0x18)) {
          *(byte *)(iVar8 + 0x1c) = *(byte *)(iVar8 + 0x1c) | 2;
          *(ushort *)(iVar6 + 0x16) = *(ushort *)(iVar6 + 0x16) | uVar10;
        }
      }
      return 0;
    }
  }
  else if ((*(ushort *)(iVar9 + 0x160) & uVar5) == 0) goto LAB_00009534;
  if ((int)((uint)*(ushort *)*param_1 << 0x18) < 0) {
    if (-1 < (int)((uint)((ushort *)*param_1)[0xc] << 0x1b)) {
      return 1;
    }
    *(ushort *)(iVar9 + 0x160) = *(ushort *)(iVar9 + 0x160) & ~uVar10;
    return 1;
  }
LAB_00009534:
  uVar10 = *(ushort *)(iVar9 + 0x15e) & ~uVar10;
  *(ushort *)(iVar9 + 0x15e) = uVar10;
  if (*(short *)(iVar9 + 0x2e) != 0) {
    *(ushort *)(iVar9 + 0x2e) =
         (~uVar1 | *(ushort *)(iVar9 + 0x160) | uVar10) & *(ushort *)(iVar9 + 0x2c);
  }
  return 1;
}



/* ======================================================================
 * 00009550  txp_pipe_tx_done_retry
 * ====================================================================== */

/* txp_pipe_tx_done_retry(pipe_mask, ...) -- the per-pipe TX-done and retry path.
   Runs when a pipe's current descriptor finishes; decides retry vs give-up and
   re-programs the hardware.  Note Ghidra already types the frame argument as
   xr_tx_pas* here, which is independent confirmation of the tx_ctx+0x54 view.
   
   Flow:
     * slot state 3 -> 4, mark the MAC busy flag.
     * bump the per-subframe retry nibble for every frame in the A-MPDU chain
       (walks pas->dwNextInAmpdu, saturating each 4-bit counter at 15).
     * pas_tx_retry_advance(pas, gave_up):
         returns 0  -> out of retries: walk the chain calling txp_fn_2441 to
                       complete each subframe, reset the pipe slot, done.
         returns !0 -> txp_program_pipe_hw(); on success re-arm, otherwise requeue
                       via tx_requeue_after_ps.
     * on a rate change (flags bit 0x80000) it rebuilds the rate words for every
       subframe (txp_desc_set_rate / _rate2) and, if RTS/CTS is enabled, rebuilds
       the protection descriptor via txp_build_pipe_words_rts.
   
   *** THIS IS WHERE THE A-MPDU COUNTERS COME FROM (MIB 0x1036). ***
   When the pipe descriptor is an aggregate (`*head == 1`):
   
     g_ampdu_ctrs[0x04] += 1;                 /* countTxAMPDUs           */
     g_ampdu_ctrs[0x08] += subframe_count;    /* countTxMPDUsInAMPDUs    */
   
   where subframe_count is obtained by walking dwNextInAmpdu.  Dividing the two
   gives the achieved mean aggregation depth, which is the single most useful number
   for the throughput question -- it says directly whether aggregates are being
   built deep or collapsing to 1-2 subframes.  Sample both before and after a slow
   run; see DRIVER-API-NOTES.md for the read path.
   
   Also increments, for rate-code 0x0C results, DAT_000098C8 + 0x24 / + 0x28 (split
   by a status byte) and DAT_000098CC + 0x28 -- per-pipe give-up counters. */

void txp_pipe_tx_done_retry(uint param_1,undefined4 param_2,undefined4 param_3,int param_4)

{
  uint *puVar1;
  byte bVar2;
  byte bVar3;
  byte bVar4;
  byte *pbVar5;
  int iVar6;
  byte *pbVar7;
  undefined4 uVar8;
  xr_tx_pas *pxVar9;
  byte *pbVar10;
  uint uVar11;
  char *pcVar12;
  uint uVar13;
  uint uVar14;
  undefined4 uVar15;
  uint uVar16;
  int iVar17;
  xr_tx_pas *pas;
  int iVar18;
  undefined4 *puVar19;
  undefined4 local_20;
  int local_1c;
  int local_18;
  
  iVar6 = DAT_000098c4;
  pbVar5 = DAT_000098c0;
  local_1c = 0;
  local_20 = 0;
  bVar2 = *DAT_000098c0;
  iVar18 = (uint)bVar2 * 0x6c + DAT_000098c4;
  *(int *)(DAT_000098c0 + 0xc) = iVar18 + 0xa0;
  iVar17 = (uint)*(byte *)(iVar18 + 0xa2) * 0x18 + iVar18 + 0xa0;
  uVar11 = 0x100 << (uint)bVar2;
  *(char **)(pbVar5 + 0x10) = (char *)(iVar17 + 0xc);
  pas = *(xr_tx_pas **)(iVar17 + 0x18);
  if ((uVar11 & param_1) == 0) {
    return;
  }
  if (*(char *)(iVar17 + 0xf) != '\x03') {
    return;
  }
  *(undefined1 *)(iVar17 + 0xf) = 4;
  *(undefined1 *)(iVar6 + 7) = 1;
  uVar8 = DAT_00009ca0;
  if (*(char *)(iVar18 + 0xa3) != '\x01') {
    uVar16 = 0;
    do {
      if ((0x100 << (uVar16 & 0xff) & param_1) != 0) {
        *(undefined4 *)(*(int *)(uVar16 * 0x6c + iVar6 + 0xa8) + 0x18) = uVar8;
      }
      uVar16 = uVar16 + 1;
    } while (uVar16 < 4);
    iVar6 = -(uVar11 + 1);
    iVar17 = DAT_00009cb0;
    goto LAB_00009a2e;
  }
  local_18 = param_4;
  if ((int)(pas->dwFlags << 0x1b) < 0) {
LAB_000095e2:
    iVar6 = *(int *)(DAT_000098c0 + 0x10);
    if (*(char *)(iVar6 + 1) == '\f') {
      if (*(char *)(iVar6 + 2) == '\0') {
        *(int *)(DAT_000098c8 + 0x28) = *(int *)(DAT_000098c8 + 0x28) + 1;
      }
      else {
        *(int *)(DAT_000098c8 + 0x24) = *(int *)(DAT_000098c8 + 0x24) + 1;
      }
    }
    if (*(char *)(iVar6 + 1) == '\x06') goto LAB_00009608;
    if (local_1c == 0) {
      uVar11 = (uint)(pas->bRateIdx >> 3);
      iVar6 = (pas->bRateIdx & 7) << 2;
      local_18 = uVar11 * 4;
      pxVar9 = pas;
      do {
        uVar16 = (&pxVar9->dwStatus)[uVar11] >> iVar6 & 0xf;
        (&pxVar9->dwStatus)[uVar11] =
             (uVar16 < 0xf) + uVar16 << iVar6 | (&pxVar9->dwStatus)[uVar11] & ~(0xf << iVar6);
        pxVar9 = (xr_tx_pas *)pxVar9->dwNextInAmpdu;
      } while (pxVar9 != (xr_tx_pas *)0x0);
    }
  }
  else {
    if (*(char *)(iVar17 + 0xd) != '\x06') {
      pas->dwFlags = pas->dwFlags | 0x10;
      pxVar9 = pas;
      if (*(char *)(iVar17 + 0xc) == '\x01') {
        do {
          uVar16 = pxVar9->dwParentQ;
          uVar11 = *(ushort *)&pxVar9->field_0xa | 0x800;
          *(uint *)(uVar16 + 0xc) = uVar11 + 0x31000000;
          *(uint *)(uVar16 + 0x10) = (uVar11 >> 8) + 0x47000000;
          puVar1 = &pxVar9->dwNextInAmpdu;
          pxVar9 = (xr_tx_pas *)*puVar1;
        } while ((xr_tx_pas *)*puVar1 != (xr_tx_pas *)0x0);
      }
      else {
        txp_submit_to_pipe(*(int *)(iVar17 + 0x20) + 0xc,pas,pas->wDurAck);
      }
      goto LAB_000095e2;
    }
LAB_00009608:
    pbVar5 = DAT_000098c0;
    *(int *)(DAT_000098cc + 0x28) = *(int *)(DAT_000098cc + 0x28) + 1;
    *(int *)(pbVar5 + 8) = *(int *)(pbVar5 + 8) + 1;
    local_1c = 1;
  }
  pbVar5 = DAT_000098c0;
  if (*(char *)(*(int *)(DAT_000098c0 + 0xc) + 5) < '\x01') {
    *(undefined1 *)(*(int *)(DAT_000098c0 + 0xc) + 5) = 1;
  }
  iVar6 = pas_tx_retry_advance(pas,local_1c);
  if (iVar6 == 0) {
    *(int *)(DAT_000098d0 + 0x18) = (1 << *pbVar5) << 0x19;
    iVar6 = *(int *)(pbVar5 + 0xc);
    puVar19 = (undefined4 *)(*(int *)(iVar6 + 8) + 0x1c);
    if (*(char *)(*(int *)(pbVar5 + 0x10) + 1) == '\x06') {
      uVar11 = (uint)*(byte *)(iVar6 + 2);
      *puVar19 = 1;
      do {
        uVar11 = uVar11 + 1 & 3;
        iVar6 = uVar11 * 0x18 + *(int *)(pbVar5 + 0xc);
        iVar17 = iVar6 + 0xc;
        *(int *)(pbVar5 + 0x10) = iVar17;
        txp_fn_2441(*(undefined4 *)(iVar6 + 0x18),iVar17,0xb);
        *puVar19 = 1;
      } while (*(byte *)(*(int *)(pbVar5 + 0xc) + 1) != uVar11);
      pbVar5[1] = 1;
      pbVar7 = *(byte **)(pbVar5 + 0xc);
      *pbVar7 = pbVar7[1] + 1 & 3;
    }
    else {
      bVar2 = *(byte *)(iVar6 + 2);
      bVar3 = *(byte *)(iVar6 + 1);
      iVar6 = (uint)bVar2 * 0x18 + iVar6;
      iVar17 = iVar6 + 0xc;
      *(int *)(pbVar5 + 0x10) = iVar17;
      txp_fn_2441(*(undefined4 *)(iVar6 + 0x18),iVar17,0xb);
      *puVar19 = 1;
      pbVar7 = *(byte **)(pbVar5 + 0xc);
      bVar4 = pbVar7[2] + 1 & 3;
      pbVar7[2] = bVar4;
      *pbVar7 = bVar4;
      if ((uint)bVar2 != (uint)bVar3) goto LAB_00009724;
    }
    pbVar7[3] = 0;
    pbVar7[4] = 0;
    pbVar7[5] = 5;
LAB_00009724:
    *(int *)(DAT_000098d0 + 4) = -((0x100 << *pbVar5) + 0x10);
    return;
  }
  iVar6 = txp_program_pipe_hw(pas,&local_20);
  iVar17 = DAT_000098d0;
  if (iVar6 == 0) {
    iVar6 = *(int *)(pbVar5 + 0xc);
    uVar11 = (uint)*(byte *)(iVar6 + 2);
    *(int *)(DAT_000098d0 + 0x18) = (1 << *pbVar5) << 0x19;
    puVar19 = (undefined4 *)(*(int *)(iVar6 + 8) + 0x1c);
    if (*(char *)(*(int *)(pbVar5 + 0x10) + 1) == '\x06') {
      iVar6 = uVar11 * 0x18 + iVar6;
      *(int *)(pbVar5 + 0x10) = iVar6 + 0xc;
      *(undefined4 *)(iVar6 + 0x18) = 0;
      uVar11 = uVar11 + 1 & 3;
      *puVar19 = 1;
    }
    while( true ) {
      iVar6 = uVar11 * 0x18 + *(int *)(pbVar5 + 0xc);
      pcVar12 = (char *)(iVar6 + 0xc);
      *(char **)(pbVar5 + 0x10) = pcVar12;
      iVar6 = *(int *)(iVar6 + 0x18);
      if (*pcVar12 == '\x01') {
        txp_fn_2441(iVar6,pcVar12,0xb);
      }
      else {
        *(uint *)(iVar6 + 4) = *(uint *)(iVar6 + 4) & 0xfff7ffff;
        tx_requeue_after_ps(iVar6,local_20);
        *(undefined4 *)(*(int *)(pbVar5 + 0x10) + 0xc) = 0;
      }
      *puVar19 = 1;
      iVar6 = DAT_000098d0;
      if (*(byte *)(*(int *)(pbVar5 + 0xc) + 1) == uVar11) break;
      uVar11 = uVar11 + 1 & 3;
    }
    pbVar7 = *(byte **)(pbVar5 + 0xc);
    bVar2 = pbVar7[1] + 1 & 3;
    *pbVar7 = bVar2;
    pbVar7[2] = bVar2;
    pbVar7[3] = 0;
    pbVar7[4] = 0;
    pbVar7[5] = 5;
    *(int *)(iVar6 + 4) = -((0x100 << *pbVar5) + 0x10);
    return;
  }
  *(int *)(DAT_000098d0 + 0x18) = (1 << *DAT_000098c0) << 0x19;
  if ((int)(pas->dwFlags << 0xc) < 0) {
    pas->dwFlags = pas->dwFlags & 0xfff7ffff;
    pcVar12 = *(char **)(DAT_000098c0 + 0x10);
    if (*pcVar12 == '\x01') {
      txp_desc_set_rate2(*(undefined4 *)(pcVar12 + 0x14),pas,0);
      pxVar9 = pas;
      do {
        pxVar9->bRateIdx = pas->bRateIdx;
        txp_desc_set_rate(pxVar9->dwParentQ,pas,0);
        pxVar9 = (xr_tx_pas *)pxVar9->dwNextInAmpdu;
      } while (pxVar9 != (xr_tx_pas *)0x0);
    }
    else {
      txp_submit_to_pipe(*(int *)(pcVar12 + 0x14) + 0xc,pas,pas->wDurAck);
    }
    if (*(char *)(*(int *)(DAT_000098c0 + 0x10) + 1) != '\x06') {
      uVar11 = pas->dwFlags;
      if ((int)(uVar11 << 0x15) < 0) {
        iVar6 = (uint)*(byte *)(DAT_000098d4 + (uint)*(byte *)(*(int *)(DAT_000098c0 + 0xc) + 2)) *
                0x18;
        txp_build_pipe_words
                  (*(int *)(iVar6 + *(int *)(DAT_000098c0 + 0xc) + 0x20) + 0xc,pas,
                   pas->wDurPreB + pas->wDurPayA + pas->wDurAck);
        tx_build_duration_desc(*(undefined4 *)(iVar6 + *(int *)(DAT_000098c0 + 0xc) + 0x20),pas,5);
        iVar6 = *(int *)(iVar6 + *(int *)(DAT_000098c0 + 0xc) + 0x20);
        *(uint *)(iVar6 + 4) = *(uint *)(iVar6 + 4) | 0x86;
      }
      else if ((int)(uVar11 << 0x10) < 0) {
        iVar6 = *(int *)(DAT_000098c0 + 0xc);
        uVar16 = (uint)*(byte *)(DAT_000098d4 + (uint)*(byte *)(iVar6 + 2));
        if ((int)(uVar11 << 0xd) < 0) {
          pas->dwFlags = uVar11 & 0xffff7fff;
          tx_build_duration_desc(*(undefined4 *)(uVar16 * 0x18 + iVar6 + 0x20),pas,2);
        }
        else {
          txp_build_pipe_words_rts
                    (*(int *)(uVar16 * 0x18 + iVar6 + 0x20) + 0xc,pas,pas->wDurPayA + pas->wDurAck);
        }
      }
    }
  }
  pbVar5 = DAT_00009c94;
  pcVar12 = *(char **)(DAT_00009c94 + 0x10);
  if ((*pcVar12 == '\x01') || (pcVar12[1] != '\x06')) {
    uVar8 = *(undefined4 *)(pcVar12 + 0x14);
    uVar15 = 1;
  }
  else {
    uVar8 = *(undefined4 *)(pcVar12 + 0x14);
    uVar15 = 5;
  }
  tx_build_duration_desc(uVar8,pas,uVar15);
  desc_or_flags(*(undefined4 *)(*(int *)(pbVar5 + 0x10) + 0x14),
                *(undefined1 *)(*(int *)(pbVar5 + 0x10) + 1));
  iVar6 = DAT_00009c98;
  if (**(char **)(pbVar5 + 0x10) == '\x01') {
    uVar11 = 0;
    *(int *)(DAT_00009c98 + 4) = *(int *)(DAT_00009c98 + 4) + 1;
    pxVar9 = pas;
    do {
      pxVar9 = (xr_tx_pas *)pxVar9->dwNextInAmpdu;
      uVar11 = uVar11 + 1 & 0xff;
    } while (pxVar9 != (xr_tx_pas *)0x0);
    *(uint *)(iVar6 + 8) = *(int *)(iVar6 + 8) + uVar11;
  }
  pbVar7 = DAT_00009c94;
  pbVar10 = *(byte **)(pbVar5 + 0xc);
  if ((int)((uint)*(byte *)(DAT_00009c9c + 0xc) << 0x1e) < 0) {
    *(undefined4 *)(*(int *)(pbVar10 + 8) + 0x18) = DAT_00009ca0;
    pbVar7 = pbVar5;
LAB_000099cc:
    bVar2 = *pbVar7;
  }
  else {
    uVar11 = (uint)pbVar10[2];
    if (*pbVar10 == uVar11) {
      iVar6 = *(int *)(pbVar10 + 8);
      uVar16 = *(uint *)(iVar6 + 0x20);
      if (pas->bFrameKind != 0xff) {
        desc_or_flags(*(undefined4 *)(pbVar10 + uVar11 * 0x18 + 0x20),pbVar10[uVar11 * 0x18 + 0xd]);
      }
      pbVar5 = DAT_00009c94;
      *(uint *)(iVar6 + 0x20) = uVar16 & ~(1 << *(sbyte *)(*(int *)(DAT_00009c94 + 0xc) + 2));
      iVar6 = -((0x100 << *pbVar5) + 1);
      goto LAB_00009a2e;
    }
    uVar16 = (uint)*(byte *)(DAT_00009ca4 + uVar11);
    if ((*(xr_tx_pas **)(pbVar10 + uVar16 * 0x18 + 0x18) != pas) ||
       ((pbVar10[uVar16 * 0x18 + 0xd] != 6 && (-1 < (int)(pas->dwFlags << 0x10))))) {
      *(uint *)(*(int *)(pbVar10 + 8) + 0x20) =
           *(uint *)(*(int *)(pbVar10 + 8) + 0x20) & ~(1 << uVar11);
      goto LAB_000099cc;
    }
    uVar13 = DAT_00009ca8 & ~(1 << uVar11) & ~(1 << uVar16) &
             *(uint *)(*(int *)(pbVar10 + 8) + 0x20);
    uVar14 = uVar16 * 0x8000000 + uVar13;
    if ((uVar13 & 0x3ffffff) >> 0x18 == uVar16) {
      uVar14 = uVar14 | 0x80000000;
    }
    pbVar10[2] = *(byte *)(DAT_00009ca4 + uVar11);
    *(uint *)(*(int *)(pbVar10 + 8) + 0x20) = uVar14;
    bVar2 = *pbVar7;
  }
  iVar6 = -((0x100 << (uint)bVar2) + DAT_00009cac);
LAB_00009a2e:
  *(int *)(iVar17 + 4) = iVar6;
  return;
}



/* ======================================================================
 * 00009a32  txp_pipe_tx_status
 * ====================================================================== */

void txp_pipe_tx_status(uint param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  byte bVar1;
  int iVar2;
  byte *pbVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  
  iVar2 = DAT_00009cb4;
  pbVar3 = DAT_00009c94;
  bVar1 = *DAT_00009c94;
  iVar5 = (uint)bVar1 * 0x6c + DAT_00009cb4;
  *(int *)(DAT_00009c94 + 0xc) = iVar5 + 0xa0;
  iVar6 = (uint)*(byte *)(iVar5 + 0xa2) * 0x18 + iVar5 + 0xa0;
  *(char **)(pbVar3 + 0x10) = (char *)(iVar6 + 0xc);
  iVar4 = *(int *)(iVar6 + 0x18);
  if ((((*(char *)(iVar5 + 0xa3) == '\x01') && (*(byte *)(iVar6 + 0xd) == param_1)) &&
      (*(char *)(iVar6 + 0xf) == '\x03')) &&
     (*(undefined1 *)(iVar6 + 0xf) = 5, *(char *)(iVar2 + 7) == '\0')) {
    if (*(char *)(iVar5 + 0xa5) < '\x01') {
      *(undefined1 *)(iVar5 + 0xa5) = 1;
    }
    if (*(char *)(iVar6 + 0xc) == '\x02') {
      *(int *)(DAT_00009cbc + 0x24) = *(int *)(DAT_00009cbc + 0x24) + 1;
      *(int *)(pbVar3 + 4) = *(int *)(pbVar3 + 4) + 1;
      *(byte *)(iVar5 + 0xa2) = *(char *)(iVar5 + 0xa2) + 1U & 3;
      return;
    }
    pas_backoff_reset(*(undefined1 *)(iVar4 + 0x69),*(undefined1 *)(DAT_00009cb8 + (uint)bVar1),
                      iVar4 + 0x60,DAT_00009cb8,param_4);
    iVar4 = *(int *)(pbVar3 + 0x10);
    iVar2 = *(int *)(iVar4 + 0xc);
    *(uint *)(iVar2 + 0x2c) = *(uint *)(iVar2 + 0x2c) | 0x200;
    txp_fn_2441(iVar2,iVar4,0);
    pbVar3 = *(byte **)(pbVar3 + 0xc);
    if (pbVar3[2] == pbVar3[1]) {
      *pbVar3 = pbVar3[1] + 1 & 3;
      pbVar3[3] = 0;
      pbVar3[4] = 0;
      pbVar3[5] = 5;
      return;
    }
    bVar1 = pbVar3[2] + 1 & 3;
    pbVar3[2] = bVar1;
    *pbVar3 = bVar1;
  }
  return;
}



/* ======================================================================
 * 00009ae6  mac_irq_tx_status_dispatch
 * ====================================================================== */

void mac_irq_tx_status_dispatch(uint param_1)

{
  int iVar1;
  uint *puVar2;
  int iVar3;
  uint uVar4;
  
  iVar1 = DAT_00009cbc;
  if (param_1 - 6 < 0x13) {
    *(int *)(DAT_00009cbc + 0x2c) = *(int *)(DAT_00009cbc + 0x2c) + 1;
  }
  iVar3 = *(int *)(DAT_00009cbc + 0x20);
  if (param_1 == 4) {
    *(int *)(iVar1 + 0x14) = *(int *)(iVar1 + 0x14) + 1;
  }
  else {
    if (4 < (int)param_1) {
      if (param_1 == 0xe) {
        *(undefined4 *)(DAT_00009cc0 + 0x10) = *(undefined4 *)(DAT_00009cb4 + 0x14);
        rxfifo_find_frame_by_subtype(0x80);
        puVar2 = DAT_00009cc8;
        iVar1 = DAT_00009cc4;
        if (*(int *)(DAT_00009cc4 + 0x28) != 0) {
          uVar4 = *(uint *)(DAT_00009cc4 + -0x10) & 0xfffffffe;
          *(uint *)(DAT_00009cc4 + -0x10) = uVar4;
          *puVar2 = uVar4;
          *(undefined4 *)(iVar1 + -0x18) = 1;
        }
      }
      goto LAB_00009b14;
    }
    if (param_1 == 0) {
      *(int *)(DAT_00009c98 + 0x14) = *(int *)(DAT_00009c98 + 0x14) + 1;
      goto LAB_00009b14;
    }
    if (param_1 != 3) goto LAB_00009b14;
    *(int *)(iVar1 + 0x10) = *(int *)(iVar1 + 0x10) + 1;
  }
  *(int *)(iVar1 + 0x20) = iVar3 + 1;
LAB_00009b14:
  txp_pipe_tx_status(param_1 & 0xff);
  return;
}



/* ======================================================================
 * 00009b60  mac_program_random_backoff
 * ====================================================================== */

void mac_program_random_backoff(int param_1,int param_2)

{
  uint uVar1;
  
  uVar1 = fw_rand_masked(*(undefined4 *)
                          ((uint)*(byte *)(param_2 + 0x69) * 0x98 + DAT_00009ccc +
                           (uint)*(byte *)(param_2 + 0xc) * 4 + 0x4bc));
  *(uint *)(param_1 + 4) = (uVar1 & 0xfff) * 0x400 + (*(uint *)(param_1 + 4) & 0x1ff);
  return;
}



/* ======================================================================
 * 00009b92  mac_pipe_irq_service
 * ====================================================================== */

void mac_pipe_irq_service(uint param_1)

{
  byte *pbVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  uint uVar7;
  
  iVar5 = DAT_00009cb4;
  iVar2 = DAT_00009cb0;
  pbVar1 = DAT_00009c94;
  iVar3 = (uint)*DAT_00009c94 * 0x6c + DAT_00009cb4;
  iVar4 = iVar3 + 0xa0;
  *(int *)(DAT_00009c94 + 0xc) = iVar4;
  *(uint *)(pbVar1 + 0x10) = (uint)*(byte *)(iVar3 + 0xa2) * 0x18 + iVar4 + 0xc;
  if ((param_1 & 0xf) == 0) {
    if ((param_1 & 0xff) >> 4 == 0) {
      if ((param_1 & 0xffff) >> 0xc == 0) {
        return;
      }
      param_1 = 0x1000 << *DAT_00009c94;
    }
    else {
      uVar7 = 0;
      for (uVar6 = (param_1 & 0xff) >> 4; (uVar6 & 1) == 0; uVar6 = uVar6 >> 1) {
        uVar7 = uVar7 + 1;
      }
      **(undefined4 **)(DAT_00009cd4 + uVar7 * 4) = DAT_00009cd0;
      *(int *)(iVar2 + 0x18) = (1 << (uVar7 & 0xff)) << 0x19;
      iVar5 = uVar7 * 0x6c + iVar5;
      iVar3 = iVar5 + 0xa0;
      *(int *)(pbVar1 + 0xc) = iVar3;
      *(uint *)(pbVar1 + 0x10) = (uint)*(byte *)(iVar5 + 0xa2) * 0x18 + iVar3 + 0xc;
      param_1 = 0x10 << (uVar7 & 0xff);
    }
  }
  else {
    uVar7 = 0;
    do {
      if ((1 << (uVar7 & 0xff) & param_1) != 0) {
        iVar3 = uVar7 * 0x6c + iVar5;
        iVar3 = (uint)*(byte *)(iVar3 + 0xa2) * 0x18 + iVar3;
        mac_program_random_backoff(*(undefined4 *)(iVar3 + 0xc0),*(undefined4 *)(iVar3 + 0xb8));
      }
      uVar7 = uVar7 + 1;
    } while (uVar7 < 3);
    param_1 = param_1 & 0xf;
    *(uint *)(iVar2 + 0x18) = param_1 << 0x19;
  }
  *(uint *)(iVar2 + 4) = -(param_1 + 1);
  return;
}



/* ======================================================================
 * 00009c4e  mac_event_dispatch
 * ====================================================================== */

void mac_event_dispatch(int param_1,int param_2)

{
  int iVar1;
  
  iVar1 = DAT_00009cc4;
  if ((param_1 != 0x14) && (0x14 < param_1)) {
    if (param_1 == 0x19) {
      if (*(int *)(DAT_00009cc4 + -0x18) == 4) {
        *DAT_00009cd8 = *DAT_00009cd8 | 0x1000000;
      }
      else if (*(int *)(DAT_00009cc4 + -0x18) != 5) {
        mac_arm_beacon_tx();
      }
      *(undefined4 *)(iVar1 + -0x18) = 1;
    }
    else if (param_1 == 0x35) {
      lmc_sched_radio_state_set(param_2 != 1);
      return;
    }
  }
  return;
}



/* ======================================================================
 * 00009cdc  txp_pipe_tx_success
 * ====================================================================== */

void txp_pipe_tx_success(int param_1)

{
  byte bVar1;
  int iVar2;
  int iVar3;
  byte *pbVar4;
  int iVar5;
  uint uVar6;
  
  iVar2 = DAT_0000a0b8;
  iVar3 = param_1 * 0x6c + DAT_0000a0b8;
  pbVar4 = (byte *)(iVar3 + 0xa0);
  bVar1 = *(byte *)(iVar3 + 0xa2);
  iVar5 = *(int *)(pbVar4 + (uint)bVar1 * 0x18 + 0x18);
  *(int *)(DAT_0000a0bc + 0x18) = *(int *)(DAT_0000a0bc + 0x18) + 1;
  *(undefined1 *)(iVar2 + 7) = 0;
  pbVar4[(uint)bVar1 * 0x18 + 0xf] = 3;
  if ((*(char *)(iVar3 + 0xa3) == '\x01') && (pbVar4[(uint)bVar1 * 0x18 + 0xd] == 0xff)) {
    if (-1 < *(int *)(iVar5 + 4) << 0x1b) {
      txq_set_frame_lifetime(iVar5);
    }
    if (-1 < *(int *)(iVar5 + 4) << 0x10) {
      pas_backoff_reset(*(undefined1 *)(iVar5 + 0x69),*(undefined1 *)(DAT_0000a0c0 + param_1));
    }
    if (*(char *)(iVar3 + 0xa2) == *(char *)(iVar3 + 0xa1)) {
      for (uVar6 = (uint)*pbVar4;
          txp_fn_2441(*(undefined4 *)(pbVar4 + uVar6 * 0x18 + 0x18),pbVar4 + uVar6 * 0x18 + 0xc,0),
          *(byte *)(iVar3 + 0xa1) != uVar6; uVar6 = uVar6 + 1 & 3) {
      }
      *pbVar4 = *(char *)(iVar3 + 0xa1) + 1U & 3;
      *(undefined1 *)(iVar3 + 0xa3) = 0;
      *(undefined1 *)(iVar3 + 0xa4) = 0;
      *(undefined1 *)(iVar3 + 0xa5) = 5;
    }
    else {
      *(byte *)(iVar3 + 0xa2) = *(char *)(iVar3 + 0xa2) + 1U & 3;
    }
  }
  iVar2 = DAT_0000a0c8;
  *(undefined1 *)(DAT_0000a0c4 + param_1) = 0;
  if (*(int *)(iVar2 + 0x2c) == 4) {
    *(undefined1 *)(iVar2 + 0x40) = 3;
    phy_state_cmd_dispatch((undefined1 *)(iVar2 + 0x40),iVar2 + 0x48);
    *(undefined4 *)(iVar2 + 0x2c) = 2;
    evt_flags_set(DAT_0000a0cc,0x40000);
  }
  return;
}



/* ======================================================================
 * 00009d9e  mac_irq_count_status
 * ====================================================================== */

void mac_irq_count_status(int param_1)

{
  int iVar1;
  int iVar2;
  
  iVar2 = DAT_0000a0d4;
  if (param_1 != 0x13) {
    if (param_1 < 0x14) {
      if ((param_1 != 7) && (param_1 != 8)) {
        return;
      }
    }
    else if (param_1 != 0x14) {
      if (param_1 != 0x19) {
        return;
      }
      if (*(int *)(DAT_0000a0d0 + 0x28) != 0) {
        *(undefined4 *)(DAT_0000a0d4 + 0xc) = 1;
      }
      iVar1 = DAT_0000a0b8;
      *(undefined1 *)(DAT_0000a0b8 + 6) = 1;
      *(undefined1 *)(iVar1 + 0xc) =
           *(undefined1 *)((uint)*(byte *)(DAT_0000a0c8 + 0x58) * 0x98 + iVar2 + 0x473);
      return;
    }
  }
  *(int *)(DAT_0000a0bc + 0x18) = *(int *)(DAT_0000a0bc + 0x18) + 1;
  return;
}



/* ======================================================================
 * 00009dea  txp_pipe_tx_start
 * ====================================================================== */

void txp_pipe_tx_start(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined2 uVar1;
  int iVar2;
  int iVar3;
  int *piVar4;
  int iVar5;
  int iVar6;
  undefined1 *puVar7;
  int iVar8;
  
  iVar2 = DAT_0000a0b8;
  *(undefined4 *)(DAT_0000a0b8 + 0x40) = *(undefined4 *)(DAT_0000a0d8 + 4);
  iVar8 = DAT_0000a0c8;
  iVar5 = param_1 * 0x6c + iVar2;
  iVar6 = (uint)*(byte *)(iVar5 + 0xa2) * 0x18 + iVar5 + 0xa0;
  if (*(char *)(iVar5 + 0xa3) == '\0') {
    func_0xfff019c8(8,0,iVar2,0x18,param_4);
  }
  else {
    *(undefined1 *)(iVar6 + 0xf) = 2;
    iVar3 = DAT_0000a0dc;
    iVar2 = DAT_0000a0c8;
    piVar4 = *(int **)(iVar6 + 0x18);
    if (*(int *)(iVar8 + 0x2c) == 3) {
      puVar7 = (undefined1 *)(DAT_0000a0c8 + 0x40);
      *puVar7 = 2;
      *(undefined1 *)(iVar2 + 0x41) = *(undefined1 *)(iVar3 + (uint)*(byte *)((int)piVar4 + 0xf));
      phy_state_cmd_dispatch(puVar7,iVar2 + 0x48);
      if (*(char *)(iVar2 + 0x48) == '\x04') {
        *(undefined4 *)(iVar8 + 0x2c) = 4;
      }
      else {
        evt_flags_set(DAT_0000a0cc,0x40000);
      }
    }
    if ((((*(char *)(iVar5 + 0xa3) == '\x01') && (*(char *)(iVar6 + 0xc) == '\0')) &&
        ((*(ushort *)((int)piVar4 + 10) & 0xf) != 4)) && ((piVar4[1] & 1U) == 0)) {
      uVar1 = *(undefined2 *)((uint)*(byte *)((int)piVar4 + 0x6a) * 2 + DAT_0000a0e0 + DAT_0000a0e4)
      ;
      *(undefined2 *)(*piVar4 + 0x16) = uVar1;
      *(undefined2 *)(piVar4 + 0x15) = uVar1;
      piVar4[1] = piVar4[1] | 1;
      iVar2 = DAT_0000a0b8;
      *(undefined1 *)(DAT_0000a0b8 + 6) = 1;
      *(undefined1 *)(iVar2 + 0xc) = *(undefined1 *)((int)piVar4 + 0x6a);
    }
    if ((-1 < *(int *)(*(int *)(iVar6 + 0x20) + 8) << 4) &&
       (*(char *)(iVar5 + 0xa2) != *(char *)(iVar5 + 0xa1))) {
      *(byte *)(iVar5 + 0xa2) = *(char *)(iVar5 + 0xa2) + 1U & 3;
      return;
    }
  }
  return;
}



/* ======================================================================
 * 00009eb4  mac_irq_handler
 * ====================================================================== */

void mac_irq_handler(void)

{
  uint *puVar1;
  byte bVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  ushort *puVar7;
  uint uVar8;
  uint uVar9;
  byte *pbVar10;
  
  uVar8 = *(uint *)(DAT_0000a0e8 + 0x20);
  pbVar10 = (byte *)(DAT_0000a0c4 + -0x14);
  do {
    iVar4 = DAT_0000a0fc;
    if ((int)uVar8 < 0) {
      if (((int)((uint)*(byte *)(DAT_0000a0fc + 0xc) << 0x1e) < 0) &&
         ((iVar5 = mac_hw_idle(), iVar5 != 0 || (iVar5 = tx_pipes_all_idle(), iVar5 != 0)))) {
        puVar1 = DAT_0000a0cc;
        *(byte *)(iVar4 + 0xc) = *(byte *)(iVar4 + 0xc) & 0xfd | 4;
        *puVar1 = *puVar1 | 0x80000000;
      }
      return;
    }
    trace_push_word(uVar8);
    if ((int)(uVar8 << 1) < 0) {
      fw_assert(DAT_0000a0ec,0xde,0x29);
    }
    uVar3 = (uVar8 & 0x3fff) >> 8;
    if ((int)(uVar8 << 6) < 0) {
      if (uVar3 == 0x37) {
        *pbVar10 = (byte)((uVar8 & 0xfffff) >> 0x12);
      }
      puVar1 = DAT_0000a0cc;
      iVar4 = DAT_0000a0b8;
      uVar6 = uVar8 & 0x30000;
      if (uVar6 == 0x20000) {
        *(undefined1 *)(DAT_0000a0b8 + 6) = 0;
        if (uVar3 == 0x37) {
          txp_pipe_tx_start(*pbVar10);
        }
        else {
          mac_irq_count_status();
        }
      }
      else if ((uVar6 == 0x30000) || ((uVar6 == 0x10000 && (uVar3 == 0x19)))) {
        if (*(char *)(DAT_0000a0b8 + 10) != '\0') {
          *(undefined1 *)(DAT_0000a0b8 + 10) = 0;
          *puVar1 = *puVar1 | 0x10;
        }
        if (*(char *)(iVar4 + 6) != '\0') {
          puVar7 = (ushort *)((uint)*(byte *)(iVar4 + 0xc) * 2 + DAT_0000a0e0 + DAT_0000a0e4);
          *puVar7 = *puVar7 + 0x10 & 0xfff0;
        }
        if (uVar3 == 0x37) {
          txp_pipe_tx_success(*pbVar10);
        }
        else {
          mac_event_dispatch(uVar3,0);
        }
      }
    }
    uVar6 = *(uint *)(DAT_0000a0f0 + 4);
    if (((int)(uVar8 << 8) < 0) && ((DAT_0000a0f4 & uVar6) != 0)) {
      mac_pipe_irq_service();
    }
    if ((int)(uVar8 << 7) < 0) {
      uVar9 = uVar8 & 0x3f;
      uVar6 = uVar6 & 0x100 << (uint)*pbVar10;
      if ((uVar6 == 0) || ((uVar9 != 0x19 && (uVar9 != 4)))) {
        if ((uVar3 != 0x39) || (uVar9 != 6)) {
          mac_irq_tx_status_dispatch(uVar9);
        }
        if (((int)(uVar8 << 8) < 0) && (uVar6 != 0)) {
          uVar3 = (uint)*pbVar10;
          iVar4 = uVar3 * 0x6c + DAT_0000a0b8;
          if ((*(char *)(iVar4 + 0xa3) == '\x01') &&
             ((*(byte *)((uint)*(byte *)(iVar4 + 0xa2) * 0x18 + iVar4 + 0xad) != uVar9 &&
              (bVar2 = *(char *)(DAT_0000a0c4 + uVar3) + 1, *(byte *)(DAT_0000a0c4 + uVar3) = bVar2,
              2 < bVar2)))) goto LAB_00009f98;
        }
      }
      else {
LAB_00009f98:
        txp_pipe_tx_done_retry(uVar6);
      }
    }
    if ((int)(uVar8 << 5) < 0) {
      *(undefined4 *)(DAT_0000a0d4 + 0xc) = 0;
      mac_arm_beacon_tx();
    }
    puVar1 = DAT_0000a0cc;
    if ((int)(uVar8 << 0x18) < 0) {
      *(undefined4 *)(DAT_0000a0c8 + 0x14) = *(undefined4 *)(DAT_0000a0f8 + 0x10);
      *(int *)(DAT_0000a0bc + 0x1c) = *(int *)(DAT_0000a0bc + 0x1c) + 1;
      *puVar1 = *puVar1 | 0x80000;
    }
    *(uint *)(DAT_0000a0d0 + -4) = uVar8;
    uVar8 = *(uint *)(DAT_0000a0e8 + 0x24);
    if (-1 < (int)uVar8) {
      uVar8 = *(uint *)(DAT_0000a0e8 + 0x20);
    }
  } while( true );
}



/* ======================================================================
 * 0000a05e  txq_agg_airtime_budget
 * ====================================================================== */

undefined2 txq_agg_airtime_budget(int param_1,int param_2)

{
  return *(undefined2 *)
          (param_2 * 0x98 + DAT_0000a0d4 + (uint)*(byte *)(DAT_0000a0c0 + param_1) * 2 + 0x4e0);
}



/* ======================================================================
 * 0000a078  txq_try_append_to_aggregate
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000a104) */
/* WARNING: Removing unreachable block (ram,0x0000a104) */
/* txq_try_append_to_aggregate(pipe, frame, mode, tbl, order)
   THE A-MPDU CHAIN BUILDER.  Returns 1 if `frame` was appended.
   
   FIRST, a per-link TX-pipe state gate.  If bit `link` is set in the u16 at
   0x04003E90 (= 0x04003E78 + 0x18, the header just below the vif array), the
   function dispatches on the per-link state byte at
       g_fw_ctx + link*0x38 + 0x658          (link_set_state, 0x00007B5C)
   through the switch8 at 0x0000A104 (11 entries, clamped) AND RETURNS -- it
   never reaches the append logic below:
   
       state 0,1,3,4,5,9 -> 0x0A122: allowed unless bit 7 of arg[sp+8] is set
       state 2           -> 0x0A114: only 802.11 Action frames (FC == 0x00D0)
                                     with byte[0x19] == 0
       state 6,7,8,10    -> 0x0A128: return 0, refuse to append
   Writers of that byte are all in tx_ptcs.c (txp_build_pipe_descriptor,
   txp_fn_2441/4155/4425) plus rx_handler, which writes 7 -- i.e. "blocked".
   This is the TX pipe control state machine that gives tx_ptcs.c its name.
   
   *** CORRECTION: an earlier version of this comment described the +0x658
   *** byte as an alternative BYTE BUDGET.  It is not -- it is a state
   *** selector and its path returns without appending.
   
   OTHERWISE, the normal path.  A frame joins the current aggregate only if:
       (state & 0xF) == 2                     current aggregate is an A-MPDU
       frame[0x6C] == head[0x6C]              same link
       frame[0x0F] == head[0x0F]              *** same rate index ***
       tbl[pipe + 0x428] < 0x10               *** hard cap: 16 subframes ***
       budget == 0 || bytes + frame[0x48] <= budget
   Append: tail[0x3C] = frame; tail = frame; frame[0x3C] = 0; count++.
   Otherwise the aggregate is closed (state |= 0x80) and the frame starts a
   new one.
   
   `budget` is txq_agg_byte_budget (0x0000A05E):
       *(u16 *)(g_fw_ctx + link*0x98 + tid_map[pipe]*2 + 0x4E0)
   a per-link, per-TID limit in BYTES.  Its writer has not been located (the
   only 0x4E0 offset construction in the blob is this read).
   
   *** This function does NOT read ampdu_num (vif + 0x128). *** The subframe
   cap here is the literal 0x10.  No reader of vif+0x128 has been found
   anywhere -- TALA (0x0000D254) writes it and mirrors it to
   g_txpipe_block+0x20 (exported as TXPIPE_TABLE word 9), and that is all the
   traffic in it located so far.
   
   Note a RATE CHANGE breaks the aggregate.  With the per-retry rate fallback
   in pas_tx_retry_advance, sustained retries fragment aggregates regardless
   of any length control. */

undefined4
txq_try_append_to_aggregate(int param_1,xr_tx_pas *param_2,int param_3,int param_4,byte *param_5)

{
  byte bVar1;
  char cVar2;
  uint uVar3;
  undefined4 uVar4;
  int iVar5;
  int iVar6;
  int iVar7;
  
  bVar1 = *(byte *)(param_4 + param_1);
  uVar3 = txq_agg_airtime_budget(param_1,param_2->bIfId);
  if (((uint)*(ushort *)(DAT_0000a100 + 0x18) & 1 << (uint)param_2->bLinkId) != 0) {
    uVar3 = (uint)*(byte *)((uint)param_2->bLinkId * 0x38 + DAT_0000a0d4 + 0x658);
                    /* WARNING: Could not recover jumptable at 0x0000a104. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    if (DAT_0000a108 <= uVar3) {
      uVar3 = (uint)DAT_0000a108;
    }
    uVar4 = (*(code *)((uint)*(byte *)(uVar3 + 0xa109) * 2 + 0xa109))();
    return uVar4;
  }
  if ((int)((uint)bVar1 << 0x18) < 0) {
    return 0;
  }
  iVar5 = txop_budget_check(param_2);
  if (iVar5 == 0) {
    *(byte *)(param_4 + param_1) = *(byte *)(param_4 + param_1) | 0x80;
  }
  iVar6 = param_1 * 0x100 + param_4;
  iVar7 = param_4 + param_1;
  iVar5 = param_1 * 4 + param_4;
  if (bVar1 == 0) {
    *(char *)(param_4 + param_1) = (char)param_3;
    if (param_3 == 2) {
      *(xr_tx_pas **)(iVar6 + 8) = param_2;
      *(xr_tx_pas **)(iVar5 + 0x408) = param_2;
      param_2->dwNextInAmpdu = 0;
      param_2->dwFlags = param_2->dwFlags | 0x20;
      *(undefined1 *)(iVar7 + 0x428) = 1;
    }
    else {
      *(xr_tx_pas **)(iVar6 + 8) = param_2;
      if ((-1 < (int)((uint)*(ushort *)&param_2->field_0xa << 0x1c)) ||
         ((int)(param_2->dwFlags << 0x16) < 0)) {
        *(byte *)(param_4 + param_1) = *(byte *)(param_4 + param_1) | 0x80;
      }
    }
    *(undefined1 *)(iVar7 + 4) = 1;
    param_5[*param_5 + 1] = (byte)param_1;
    *param_5 = *param_5 + 1;
    txop_budget_consume(param_2);
    if (uVar3 != 0) {
      *(uint *)(iVar5 + 0x418) = *(int *)(iVar5 + 0x418) + param_2->dwAirtimeUs;
      return 1;
    }
    if (param_3 == 2) {
      return 1;
    }
LAB_0000a1c8:
    *(byte *)(param_4 + param_1) = *(byte *)(param_4 + param_1) | 0x80;
    return 1;
  }
  if (param_3 == 2) {
    if (((((bVar1 & 0xf) == 2) && (param_2->bLinkId == *(byte *)(*(int *)(iVar6 + 8) + 0x6c))) &&
        ((param_2->bRateIdx == *(byte *)(*(int *)(iVar6 + 8) + 0xf) &&
         (*(byte *)(iVar7 + 0x428) < 0x10)))) &&
       ((uVar3 == 0 || (*(int *)(iVar5 + 0x418) + param_2->dwAirtimeUs <= uVar3)))) {
      txop_budget_consume(param_2);
      *(uint *)(iVar5 + 0x418) = *(int *)(iVar5 + 0x418) + param_2->dwAirtimeUs;
      *(xr_tx_pas **)(*(int *)(iVar5 + 0x408) + 0x3c) = param_2;
      *(xr_tx_pas **)(iVar5 + 0x408) = param_2;
      param_2->dwNextInAmpdu = 0;
      param_2->dwFlags = param_2->dwFlags | 0x20;
      *(char *)(iVar7 + 0x428) = *(char *)(iVar7 + 0x428) + '\x01';
      return 1;
    }
  }
  else if (((((param_3 == 1) && ((bVar1 & 0xf) == 1)) &&
            ((int)((uint)*(ushort *)&param_2->field_0xa << 0x1c) < 0)) &&
           ((-1 < (int)(param_2->dwFlags << 0x16) && (*(byte *)(iVar7 + 4) < 4)))) &&
          (*(int *)(iVar5 + 0x418) + param_2->dwAirtimeUs <= uVar3)) {
    txop_budget_consume(param_2);
    *(uint *)(iVar5 + 0x418) = *(int *)(iVar5 + 0x418) + param_2->dwAirtimeUs;
    *(xr_tx_pas **)(iVar6 + (uint)*(byte *)(iVar7 + 4) * 4 + 8) = param_2;
    cVar2 = *(char *)(iVar7 + 4) + '\x01';
    *(char *)(iVar7 + 4) = cVar2;
    if (cVar2 != '\x03') {
      return 1;
    }
    if ((*(uint *)(*(int *)(iVar6 + 8) + 4) & 0xfff) >> 10 == 0) {
      return 1;
    }
    goto LAB_0000a1c8;
  }
  *(byte *)(param_4 + param_1) = *(byte *)(param_4 + param_1) | 0x80;
  return 0;
}



/* ======================================================================
 * 0000a2c0  txq_build_aggregate_lists
 * ====================================================================== */

void txq_build_aggregate_lists(uint param_1,int param_2,undefined1 *param_3)

{
  byte bVar1;
  bool bVar2;
  uint *puVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  undefined4 uVar9;
  int iVar10;
  int iVar11;
  int iVar12;
  int *piVar13;
  uint local_6c;
  uint local_60;
  byte local_58 [8];
  uint local_50;
  uint local_4c;
  uint *local_48;
  int *local_44;
  int local_40;
  int local_3c;
  int local_38;
  int local_34;
  int local_30;
  int local_2c;
  int local_28;
  int local_24;
  uint local_20;
  int local_1c;
  undefined1 *local_18;
  
  local_60 = 0x1000;
  local_20 = param_1;
  local_1c = param_2;
  local_18 = param_3;
  iVar4 = fw_read_timer();
  *local_18 = 0;
  uVar5 = 0;
  do {
    *(undefined1 *)(local_1c + uVar5) = 0;
    *(undefined4 *)(uVar5 * 4 + local_1c + 0x418) = 0;
    iVar10 = local_1c + uVar5;
    uVar5 = uVar5 + 1;
    *(undefined1 *)(iVar10 + 0x428) = 0;
    iVar10 = DAT_0000a6c8;
  } while (uVar5 < 4);
  uVar5 = 0;
  do {
    local_58[uVar5] = 0;
    uVar5 = uVar5 + 1;
  } while (uVar5 < 8);
  local_24 = DAT_0000a6cc;
  if (*(short *)(DAT_0000a6cc + 0x18) != 0) {
    local_3c = DAT_0000a6c8 + -8;
    for (uVar5 = 0; uVar5 < *(byte *)(local_3c + 0xc); uVar5 = uVar5 + 1 & 0xff) {
      iVar6 = uVar5 * 0x38 + DAT_0000a6d0;
      local_28 = iVar6 + 0x640;
      bVar1 = *(byte *)(iVar6 + 0x658);
      if (1 < bVar1) {
        if (bVar1 == 8) {
          uVar7 = 0;
          uVar8 = 0;
          iVar6 = uVar5 * 0x40 + DAT_0000a6d4;
          do {
            iVar11 = iVar6 + uVar8 * 4;
            if (*(int *)(iVar11 + 0x490) == 0) {
LAB_0000a370:
              if ((*(int *)(iVar6 + uVar7 * 4 + 0x490) != 0) && (*(int *)(iVar11 + 0x490) == 0)) {
                uVar7 = uVar8;
              }
            }
            else {
              iVar12 = iVar6 + uVar7 * 4;
              if (*(int *)(iVar12 + 0x490) == 0) {
                *(int *)(iVar12 + 0x490) = *(int *)(iVar11 + 0x490);
                uVar7 = uVar7 + 1;
                *(undefined4 *)(iVar11 + 0x490) = 0;
                goto LAB_0000a370;
              }
            }
            uVar8 = uVar8 + 1;
          } while (uVar8 < 0x10);
          iVar11 = 0xf;
          do {
            iVar12 = iVar6 + iVar11 * 4;
            local_2c = iVar12 + 0x480;
            iVar12 = *(int *)(iVar12 + 0x490);
            if ((iVar12 != 0) && ((int)(*(uint *)(iVar12 + 4) << 3) < 0)) {
              uVar7 = DAT_0000a6d8 & *(uint *)(iVar12 + 4);
              *(uint *)(iVar12 + 4) = uVar7;
              *(undefined1 *)(iVar12 + 0x53) = 0;
              uVar8 = (uint)*(ushort *)(iVar12 + 10);
              iVar12 = *(int *)(iVar12 + 0x4c);
              if ((int)(uVar7 << 0x1b) < 0) {
                uVar8 = uVar8 | 0x800;
              }
              *(uint *)(iVar12 + 0xc) = uVar8 + 0x31000000;
              *(uint *)(iVar12 + 0x10) = (uVar8 >> 8) + 0x47000000;
              pas_txq_push_global();
              *(undefined4 *)(local_2c + 0x10) = 0;
            }
            iVar11 = iVar11 + -1;
          } while (-1 < iVar11);
          link_set_state(uVar5,9);
        }
        if (*(char *)(local_28 + 0x18) == '\v') {
          bVar2 = false;
          if (*DAT_0000a6dc == 0) {
            uVar7 = 0;
            local_38 = uVar5 * 0x40 + DAT_0000a6d4;
            do {
              iVar6 = local_38 + uVar7 * 4;
              iVar11 = *(int *)(iVar6 + 0x490);
              if (iVar11 != 0) {
                *(undefined2 *)(iVar11 + 0x1c) = 0xb;
                tx_ctx_free_locked();
                *(undefined4 *)(iVar6 + 0x490) = 0;
              }
              uVar7 = uVar7 + 1;
            } while (uVar7 < 0x10);
LAB_0000a4c4:
            uVar9 = 5;
          }
          else {
            iVar6 = 0xf;
            local_30 = uVar5 * 0x40 + DAT_0000a6d4;
            do {
              iVar11 = local_30 + iVar6 * 4;
              local_34 = iVar11 + 0x480;
              iVar11 = *(int *)(iVar11 + 0x490);
              if (iVar11 != 0) {
                *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) & 0xffffffdf;
                iVar12 = pas_tx_retry_advance_mode0(iVar11);
                *(uint *)(iVar11 + 4) = *(uint *)(iVar11 + 4) & DAT_0000a6e0;
                if (iVar12 == 0) {
                  *(undefined2 *)(iVar11 + 0x1c) = 0xb;
                  tx_ctx_free_locked(iVar11);
                }
                else {
                  bVar2 = true;
                  *(undefined1 *)(iVar11 + 0x53) = 0;
                  uVar7 = *(ushort *)(iVar11 + 10) | 0x800;
                  iVar12 = *(int *)(iVar11 + 0x4c);
                  *(uint *)(iVar12 + 0xc) = uVar7 + 0x31000000;
                  *(uint *)(iVar12 + 0x10) = (uVar7 >> 8) + 0x47000000;
                  pas_txq_push_global(iVar11);
                }
                *(undefined4 *)(local_34 + 0x10) = 0;
              }
              iVar6 = iVar6 + -1;
            } while (-1 < iVar6);
            if (!bVar2) goto LAB_0000a4c4;
            uVar9 = 9;
          }
          link_set_state(uVar5,uVar9);
        }
      }
    }
  }
  local_48 = DAT_0000a6e4;
  uVar5 = *DAT_0000a6e4 & 0xff;
  local_50 = DAT_0000a6e4[1] & 0xff;
  if (uVar5 != local_50) {
    local_4c = uVar5;
    pac_phy_start_op(1);
    for (; uVar5 != local_50; uVar5 = uVar5 + 1 & 0x3f) {
      local_40 = uVar5 * 4;
      piVar13 = (int *)local_48[uVar5 + 2];
      if (piVar13 != (int *)0x0) {
        if ((((piVar13[-5] - iVar4) + DAT_0000a6e8 < 0) || ((short)piVar13[7] == 0x14)) &&
           (*(char *)((int)piVar13 - 1) == '\0')) {
          DAT_0000a6e4[uVar5 + 2] = 0;
          *(undefined2 *)(piVar13 + 7) = 10;
          tx_ctx_free_locked(piVar13);
        }
        else {
          iVar6 = txp_program_pipe_hw(piVar13,0);
          if ((iVar6 != 0) &&
             (uVar7 = (uint)*(byte *)(DAT_0000a6ec + (uint)*(byte *)(piVar13 + 3)),
             (1 << uVar7 & local_20) != 0)) {
            local_6c = 0;
            uVar8 = 8;
            if (piVar13[1] << 2 < 0) {
              uVar8 = txpipe_find_by_mac_tid
                                (*(undefined1 *)((int)piVar13 + 0x69),
                                 *(undefined1 *)((int)piVar13 + 0x52),*piVar13 + 4);
              local_6c = (uint)(*(ushort *)(*piVar13 + 0x16) >> 4);
              *(uint *)(DAT_0000a6f0 + 0x24) = (uint)*(ushort *)(uVar8 * 0x38 + iVar10 + 0x14);
            }
            local_44 = piVar13 + 0x18;
            *(char *)(piVar13 + 0x1b) = (char)uVar8;
            if (((piVar13[1] << 2 < 0) && (0xd < *(byte *)((int)piVar13 + 0xf))) &&
               ((uVar8 < 8 &&
                ((((iVar6 = uVar8 * 0x38 + iVar10, 4 < *(byte *)(iVar6 + 0x10) &&
                   ((piVar13[1] & 0x3fffffU) >> 0x14 == 0)) &&
                  ((ushort)local_58[uVar8] <
                   *(ushort *)((uint)*(byte *)((int)piVar13 + 0x69) * 0x3b0 + DAT_0000a6f4 + 0x128))
                  ) && ((local_6c < *(ushort *)(iVar6 + 0x14) + local_60 &&
                        (((uint)*(ushort *)(iVar10 + (uint)*(byte *)((int)piVar13 + 0x69) * 2) &
                         1 << *(sbyte *)((int)piVar13 + 0x52)) != 0)))))))) {
              iVar6 = txq_try_append_to_aggregate(uVar7,piVar13,2,local_1c,local_18);
              puVar3 = DAT_0000a6e4;
              if (iVar6 != 0) {
                piVar13[1] = piVar13[1] & 0xffff7fff;
                *(undefined4 *)((int)puVar3 + local_40 + 8) = 0;
                *(byte *)(local_44 + 2) = local_58[uVar8];
                bVar1 = local_58[uVar8];
                *(int **)(uVar8 * 0x40 + DAT_0000a6d4 + (uint)bVar1 * 4 + 0x490) = piVar13;
                local_58[uVar8] = bVar1 + 1;
                if (local_60 <= local_6c) {
                  local_6c = local_60;
                }
                *(ushort *)(local_24 + 0x18) =
                     *(ushort *)(local_24 + 0x18) | (ushort)(1 << (uVar8 & 0xff));
                local_60 = local_6c;
LAB_0000a688:
                piVar13[0xb] = piVar13[0xb] | 0x80;
              }
            }
            else {
              iVar6 = txq_try_append_to_aggregate(uVar7,piVar13,1,local_1c,local_18);
              puVar3 = DAT_0000a6e4;
              if (iVar6 != 0) {
                piVar13[1] = piVar13[1] & 0xffff7fff;
                *(undefined4 *)((int)puVar3 + local_40 + 8) = 0;
                goto LAB_0000a688;
              }
            }
          }
        }
      }
    }
    while ((local_4c != local_50 && (local_48[local_4c + 2] == 0))) {
      local_4c = local_4c + 1 & 0x3f;
      *local_48 = local_4c;
    }
  }
  return;
}



/* ======================================================================
 * 0000a6f8  desc_freelist_pop
 * ====================================================================== */

int desc_freelist_pop(void)

{
  int *piVar1;
  
  irq_fiq_disable_save();
  piVar1 = (int *)*DAT_0000aaf4;
  if (piVar1 != (int *)0x0) {
    *DAT_0000aaf4 = *piVar1;
  }
  irq_fiq_restore();
  return (int)piVar1;
}



/* ======================================================================
 * 0000a712  txp_build_pipe_descriptor
 * ====================================================================== */

/* txp_build_pipe_descriptor(pipe_idx_p, frame, pipe_tbl, kind) -- tx_ptcs.c
   Builds the hardware TX descriptor for one transmission and advances the
   4-deep TX pipe ring:  *pipe_idx_p = (*pipe_idx_p + 1) & 3.
   
   kind selects the frame class:
     0  single frame
     1  A-MPDU: walks the frame chain via frame[0x3C], emitting descriptor
        words through txp_desc_emit (opcodes 0..5).  Asserts tx_ptcs.c:3711
        code 0x36 if no descriptor buffer is available.
     2  chained frame, walk capped at 15 links
     3  management/other
   
   It CONSUMES an already-formed aggregate chain -- the decision about how
   many MPDUs to chain (ampdu_num) is made upstream, not here.  No throughput
   cap of its own was found in this function. */

void txp_build_pipe_descriptor(byte *param_1,int param_2,int param_3,int param_4)

{
  short *psVar1;
  short sVar2;
  undefined4 uVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  uint local_54;
  uint local_50;
  int local_4c;
  int local_48;
  int local_44;
  int local_40;
  int local_3c;
  undefined4 local_38;
  int local_34;
  uint local_30;
  int local_2c;
  int local_28;
  byte *local_24;
  int iStack_20;
  int local_1c;
  int iStack_18;
  
  local_24 = param_1;
  iStack_20 = param_2;
  local_1c = param_3;
  iStack_18 = param_4;
  local_38 = fw_read_timer();
  iVar6 = (uint)*local_24 * 0x18 + local_1c;
  *(char *)(iVar6 + 0xc) = (char)param_4;
  *(int *)(iVar6 + 0x18) = param_2;
  *(undefined1 *)(iVar6 + 0xf) = 0;
  *(undefined4 *)(iVar6 + 0x1c) = 0;
  *(undefined1 *)(iVar6 + 0xe) = 0;
  if (param_4 == 0) {
    *(uint *)(param_2 + 0x2c) = *(uint *)(param_2 + 0x2c) | 0x100;
    *(undefined4 *)(param_2 + 0x18) = local_38;
    *(undefined1 *)(iVar6 + 0xd) = *(undefined1 *)(param_2 + 0x56);
    *(undefined4 *)(param_2 + 0x3c) = 0;
    txp_submit_to_pipe(*(int *)(iVar6 + 0x20) + 0xc,param_2,*(undefined2 *)(param_2 + 0x36));
    uVar7 = (uint)(*(char *)(param_2 + 0x56) != -1);
    *(uint *)(iVar6 + 0x14) =
         (uint)*(ushort *)(param_2 + 0x3a) * 0x8000 + uVar7 * 0x2000 +
         uVar7 * *(ushort *)(param_2 + 0x36);
    tx_build_duration_desc(*(undefined4 *)(iVar6 + 0x20),param_2,uVar7);
    if (uVar7 != 0) {
      desc_or_flags(*(undefined4 *)(iVar6 + 0x20),*(undefined1 *)(param_2 + 0x56));
    }
  }
  else if (param_4 == 1) {
    iVar4 = 0;
    local_34 = 0xb4;
    local_4c = 0;
    local_30 = (uint)*(ushort *)(param_2 + 0x38);
    *(undefined1 *)(iVar6 + 0xd) = 0xff;
    local_2c = 0;
    local_44 = *(int *)(iVar6 + 0x20) + 0x18;
    local_40 = desc_freelist_pop();
    if (local_40 == 0) {
      fw_assert(DAT_0000aafc,DAT_0000aaf8,0x36);
    }
    *(int *)(iVar6 + 0x1c) = local_40;
    local_48 = *(int *)(local_40 + 4);
    txp_desc_emit(0,local_48,0,&local_44);
    local_28 = param_2 + 0x60;
    local_44 = local_48;
    iVar5 = param_2;
    do {
      *(uint *)(iVar5 + 0x2c) = *(uint *)(iVar5 + 0x2c) | 0x100;
      *(undefined4 *)(iVar5 + 0x18) = local_38;
      if ((*(uint *)(iVar5 + 4) & 0x3fffff) >> 0x14 == 0) {
        local_4c = 1;
      }
      if (*(int *)(iVar5 + 0x4c) != 0) {
        local_48 = *(int *)(iVar5 + 0x4c);
      }
      txp_desc_emit(0,local_48 + 8,0,&local_44);
      if (*(int *)(iVar5 + 0x3c) == 0) {
        iVar4 = (uint)*(ushort *)(iVar5 + 8) + iVar4 + 8;
      }
      else {
        local_3c = (*(ushort *)(iVar5 + 8) + 0xb & 0xfffffffc) + iVar4;
        txp_desc_emit(1,0,0,&local_44);
        uVar7 = (uint)*(byte *)((uint)*(byte *)(param_2 + 0xf) + DAT_0000ab00 +
                               (uint)*(byte *)((uint)*(byte *)(local_28 + 9) * 0x98 + DAT_0000ab04 +
                                              0x490) * 8 + -0xe);
        txp_desc_emit(4,uVar7,0,&local_44);
        iVar4 = uVar7 * 4 + local_3c;
      }
      iVar5 = *(int *)(iVar5 + 0x3c);
    } while (iVar5 != 0);
    txp_desc_emit(2,0,0,&local_44);
    local_44 = *(int *)(iVar6 + 0x20) + 0xc;
    uVar3 = pas_rate_to_hw_code(*(undefined1 *)(param_2 + 0xf));
    pas_build_phy_rate_words
              (&local_50,&local_54,*(undefined1 *)(param_2 + 0xf),*(undefined4 *)(param_2 + 4),
               *(undefined1 *)(param_2 + 0xd));
    if ((local_54 & 0x1fff) >> 10 == 5) {
      uVar7 = pac_phy_calc_duration(*(undefined1 *)(param_2 + 0xf),*(ushort *)(param_2 + 8) + 4);
      local_50 = (uVar7 & 0xfff) << 0xc | local_50;
    }
    txp_desc_emit(3,1,local_54,&local_44);
    txp_desc_emit(3,0,local_50,&local_44);
    txp_desc_emit(5,uVar3,iVar4,&local_44);
    if (local_4c != 0) {
      *(undefined1 *)(iVar6 + 0xd) = 0xc;
      *(undefined1 *)(iVar6 + 0xe) = 1;
      local_2c = 1;
    }
    *(uint *)(iVar6 + 0x14) = local_30 * 0x8000 + local_2c * 0x2000 + local_34 * local_2c;
    *(uint *)(param_2 + 0x48) = local_30 + local_34 + 0x20;
    tx_build_duration_desc(*(undefined4 *)(iVar6 + 0x20),param_2,local_2c);
    if (local_2c != 0) {
      desc_or_flags(*(undefined4 *)(iVar6 + 0x20),*(undefined1 *)(iVar6 + 0xd));
    }
    if ((local_4c == 0) && ((*(byte *)(DAT_0000ab08 + 0x10) & 1) != 0)) {
      link_set_state(*(undefined1 *)(local_28 + 0xc),7);
    }
  }
  else if (param_4 == 2) {
    *(undefined1 *)(iVar6 + 0xd) = 6;
    sVar2 = *(short *)(param_2 + 0x38) + *(short *)(param_2 + 0x34);
    iVar4 = *(int *)(param_2 + 0x3c);
    iVar5 = 0;
    do {
      if (iVar4 == 0) break;
      psVar1 = (short *)(iVar4 + 0x3a);
      iVar5 = iVar5 + 1;
      iVar4 = *(int *)(iVar4 + 0x3c);
      sVar2 = *psVar1 + sVar2;
    } while (iVar5 < 0xf);
    txp_build_pipe_words(*(int *)(iVar6 + 0x20) + 0xc,param_2,*(short *)(param_2 + 0x36) + sVar2);
    *(uint *)(iVar6 + 0x14) =
         (uint)*(ushort *)(param_2 + 0x32) * 0x8000 + 0xa000 + (uint)*(ushort *)(param_2 + 0x34);
    tx_build_duration_desc(*(undefined4 *)(iVar6 + 0x20),param_2,5);
    *(uint *)(*(int *)(iVar6 + 0x20) + 4) = *(uint *)(*(int *)(iVar6 + 0x20) + 4) | 0x86;
  }
  else if (param_4 == 3) {
    *(undefined1 *)(iVar6 + 0xd) = 0xff;
    txp_build_pipe_words_rts
              (*(int *)(iVar6 + 0x20) + 0xc,param_2,
               *(short *)(param_2 + 0x36) + *(short *)(param_2 + 0x38));
    *(uint *)(iVar6 + 0x14) = (uint)*(ushort *)(param_2 + 0x32) << 0xf;
    tx_build_duration_desc(*(undefined4 *)(iVar6 + 0x20),param_2,0);
  }
  *(byte *)(local_1c + 1) = *local_24;
  *local_24 = *local_24 + 1 & 3;
  return;
}



/* ======================================================================
 * 0000a9f2  txp_pipe_advance_slot
 * ====================================================================== */

undefined4 txp_pipe_advance_slot(uint param_1)

{
  uint *puVar1;
  int iVar2;
  uint uVar3;
  undefined4 uVar4;
  
  uVar4 = 0;
  irq_fiq_disable_save();
  iVar2 = param_1 * 0x6c + DAT_0000ab0c;
  if (*(char *)(iVar2 + 0xa3) == '\0') {
    uVar3 = (uint)*(byte *)(iVar2 + 0xa0);
  }
  else {
    *(undefined1 *)(iVar2 + 0xa3) = 0;
    uVar4 = 1;
    if (*(char *)(iVar2 + 0xa6) != -1) {
      *(char *)(iVar2 + 0xa6) = *(char *)(iVar2 + 0xa6) + '\x01';
    }
    uVar3 = *(byte *)(iVar2 + 0xa1) + 1 & 3;
  }
  irq_fiq_restore();
  *(undefined4 *)(*(int *)(iVar2 + 0xa8) + 0x18) = DAT_0000ab10;
  *(int *)(DAT_0000ab18 + 4) = -((DAT_0000ab14 << (param_1 & 0xff)) + 0x10);
  puVar1 = (uint *)(*(int *)(iVar2 + 0xa8) + 0x20);
  *puVar1 = *puVar1 & 0xc0ffffff | uVar3 << 0x18 | uVar3 << 0x1b;
  return uVar4;
}



/* ======================================================================
 * 0000aa5e  txp_scheduler_run
 * ====================================================================== */

void txp_scheduler_run(void)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  byte *pbVar8;
  byte bVar9;
  int local_464;
  uint local_460;
  uint local_45c;
  byte local_458;
  byte abStack_457 [7];
  byte abStack_450 [8];
  int aiStack_448 [265];
  uint local_24;
  byte *local_20;
  int local_1c;
  int local_18;
  
  iVar4 = DAT_0000ab1c;
  bVar9 = 0;
  uVar7 = 0;
  if (((int)((uint)*(byte *)(DAT_0000ab1c + 0xc) << 0x1e) < 0) &&
     ((iVar1 = mac_hw_idle(), iVar1 != 0 || (iVar1 = tx_pipes_all_idle(), iVar1 != 0)))) {
    *(byte *)(iVar4 + 0xc) = *(byte *)(iVar4 + 0xc) & 0xfd | 4;
  }
  if ((int)((uint)*(byte *)(iVar4 + 0xc) << 0x1d) < 0) {
    txp_fn_4155();
  }
  if ((*(char *)(iVar4 + 0xc) == '\0') &&
     (local_18 = DAT_0000ab20, *(char *)(DAT_0000ab20 + 0x15) == '\0')) {
    uVar2 = 0;
    do {
      iVar4 = uVar2 * 0x6c + DAT_0000ab0c;
      if (*(char *)(iVar4 + 0xa3) == '\0') {
        uVar7 = 1 << (uVar2 & 0xff) & 0xffU | uVar7;
      }
      uVar2 = uVar2 + 1;
      bVar9 = bVar9 | *(byte *)(iVar4 + 0xa4) & 8;
    } while (uVar2 < 4);
    if ((bVar9 == 0) || (iVar4 = txp_fn_4425(0,0,1), iVar4 == 0)) {
      iVar4 = DAT_0000ab24;
      if ((*(byte *)(DAT_0000ab24 + 0x19) & 1) != 0) {
        if (uVar7 != 0xf) {
          return;
        }
        phy_rx_disable_and_drain();
        pac_phy_start_op(6);
        *(byte *)(iVar4 + 0x19) = *(byte *)(iVar4 + 0x19) & 0xfe;
        phy_rx_enable();
      }
      txq_build_aggregate_lists(uVar7,abStack_450,&local_458);
      local_1c = DAT_0000adb4;
      for (local_45c = 0; local_45c < local_458; local_45c = local_45c + 1) {
        local_460 = 0;
        uVar7 = (uint)abStack_457[local_45c];
        iVar4 = uVar7 * 0x6c + DAT_0000adb8;
        pbVar8 = (byte *)(iVar4 + 0xa0);
        local_24 = (uint)*pbVar8;
        *(byte *)(iVar4 + 0xa2) = *pbVar8;
        if ((abStack_450[uVar7] & 0xf) == 2) {
          iVar1 = aiStack_448[uVar7 * 0x40];
          *(uint *)(iVar1 + 4) = *(uint *)(iVar1 + 4) | 0x40;
          if (*(byte *)(iVar1 + 0x6c) < 8) {
            link_set_state(*(byte *)(iVar1 + 0x6c),6);
          }
          if (*(int *)(iVar1 + 4) << 0x15 < 0) {
            txp_build_pipe_descriptor(&local_24,iVar1,pbVar8,2);
          }
          txp_build_pipe_descriptor(&local_24,aiStack_448[uVar7 * 0x40],pbVar8,1);
          *DAT_0000adbc = *DAT_0000adbc + 1;
          local_460 = *(uint *)(iVar1 + 0x48) & 0xffff;
LAB_0000ac7a:
          local_24 = (uint)*pbVar8;
          *(undefined4 *)(*(int *)(iVar4 + 0xa8) + 0x14) = 0;
          iVar6 = *(int *)(pbVar8 + local_24 * 0x18 + 0x18);
          iVar1 = (uint)*(byte *)(iVar6 + 0x69) * 0x98 + DAT_0000adc0;
          iVar5 = *(int *)(iVar1 + 0x4fc);
          uVar2 = (uint)*(ushort *)(iVar1 + (uint)*(byte *)(DAT_0000adc4 + uVar7) * 2 + 0x4e0);
          if (*(int *)(local_1c + 4) != iVar5) {
            *(int *)(DAT_0000adc8 + 0x24) = iVar5;
            *(int *)(local_1c + 4) = iVar5;
          }
          if (uVar2 == 0) {
            if ((*(uint *)(iVar6 + 4) & 0xfff) >> 10 != 0) {
LAB_0000acd4:
              uVar2 = local_460;
            }
          }
          else if (uVar2 <= local_460) {
            *(ushort *)(iVar6 + 0x50) = *(ushort *)(iVar6 + 0x50) | 8;
            goto LAB_0000acd4;
          }
          **(uint **)(DAT_0000adcc + uVar7 * 4) = uVar2 + 0x1f >> 5;
          *(int *)(DAT_0000adc8 + 0x58) = (1 << uVar7) << 0x19;
          while( true ) {
            uVar7 = local_24;
            iVar1 = *(int *)(pbVar8 + local_24 * 0x18 + 0x18);
            *(char *)(local_18 + 0x14) = *(char *)(local_18 + 0x14) + '\x01';
            trace_push_pair(*(undefined2 *)(iVar1 + 10),*(undefined2 *)(iVar1 + 8));
            pbVar8[uVar7 * 0x18 + 0xf] = 1;
            **(undefined4 **)(iVar4 + 0xa8) = *(undefined4 *)(pbVar8 + uVar7 * 0x18 + 0x14);
            if ((uint)*(byte *)(iVar4 + 0xa1) == (local_24 & 0xff)) break;
            local_24 = (local_24 & 0xff) + 1 & 3;
          }
          *(undefined1 *)(iVar4 + 0xa3) = 1;
          *(byte *)(iVar4 + 0xa4) = *(byte *)(iVar4 + 0xa4) | 1;
          *(undefined1 *)(iVar4 + 0xa5) = 5;
          *(undefined4 *)(*(int *)(iVar4 + 0xa8) + 0x14) = 1;
        }
        else {
          if ((abStack_450[uVar7] & 0xf) != 1) goto LAB_0000ac7a;
          local_464 = 0;
          local_20 = abStack_450 + uVar7;
          for (uVar2 = 0; uVar2 < local_20[4]; uVar2 = uVar2 + 1) {
            iVar1 = aiStack_448[uVar7 * 0x40 + uVar2];
            local_464 = local_464 + 1;
            if (local_464 == 1) {
              pas_policy_retime_if_mode2(iVar1);
              local_460 = *(uint *)(iVar1 + 0x48) & 0xffff;
              uVar3 = *(uint *)(iVar1 + 4);
              *(uint *)(iVar1 + 4) = uVar3 | 0x4000000;
              if ((int)(uVar3 << 0x15) < 0) {
                txp_build_pipe_descriptor(&local_24,iVar1,pbVar8,2);
              }
              else if ((int)(uVar3 << 0x14) < 0) {
                txp_build_pipe_descriptor(&local_24,iVar1,pbVar8,3);
                *(uint *)(iVar1 + 4) = *(uint *)(iVar1 + 4) | 0x8000;
              }
            }
            else {
              *(uint *)(iVar1 + 4) = *(uint *)(iVar1 + 4) | 0x8000000;
              pas_policy_retime_if_mode2(iVar1);
            }
            txp_build_pipe_descriptor(&local_24,iVar1,pbVar8,0);
          }
          if (local_464 != 0) goto LAB_0000ac7a;
        }
      }
    }
  }
  return;
}



/* ======================================================================
 * 0000ad76  txp_abort_all_pipes
 * ====================================================================== */

void txp_abort_all_pipes(void)

{
  int iVar1;
  undefined4 uVar2;
  uint uVar3;
  int iVar4;
  
  uVar2 = mac_rx_pause();
  iVar1 = DAT_0000adb8;
  uVar3 = 0;
  do {
    iVar4 = uVar3 * 0x6c + iVar1;
    *(undefined4 *)(*(int *)(iVar4 + 0xa8) + 0x14) = 0;
    if (*(char *)(iVar4 + 0xa3) != '\0') {
      *(byte *)(iVar4 + 0xa4) = *(byte *)(iVar4 + 0xa4) | 8;
      *(char *)(iVar4 + 0xa7) = *(char *)(iVar4 + 0xa7) + '\x01';
    }
    uVar3 = uVar3 + 1 & 0xff;
  } while (uVar3 < 4);
  *(undefined4 *)(DAT_0000adc8 + 0x4c) = uVar2;
  return;
}



/* ======================================================================
 * 0000add0  txp_submit_to_pipe
 * ====================================================================== */

void txp_submit_to_pipe(int *param_1,uint *param_2,uint param_3)

{
  uint uVar1;
  uint *puVar2;
  int iVar3;
  uint uVar4;
  uint uVar5;
  uint uVar6;
  uint local_30;
  uint local_2c;
  int local_28;
  uint local_24;
  int *local_20;
  uint *puStack_1c;
  uint local_18;
  
  uVar4 = (uint)*(ushort *)((int)param_2 + 10);
  uVar6 = (uint)(ushort)param_2[2];
  uVar5 = param_2[1];
  local_24 = uVar4 & 0xff;
  local_20 = param_1;
  puStack_1c = param_2;
  local_18 = param_3;
  if (local_24 == 0x50) {
    txp_build_beacon_pipe_words(param_1,param_2,param_3);
    return;
  }
  local_28 = pas_rate_to_hw_code(*(undefined1 *)((int)param_2 + 0xf));
  pas_build_phy_rate_words
            (&local_2c,&local_30,*(undefined1 *)((int)param_2 + 0xf),param_2[1],
             *(undefined1 *)((int)param_2 + 0xd));
  if ((local_30 & 0x1fff) >> 10 == 5) {
    uVar1 = pac_phy_calc_duration(*(undefined1 *)((int)param_2 + 0xf),(ushort)param_2[2] + 4);
    local_2c = (uVar1 & 0xfff) << 0xc | local_2c;
  }
  *local_20 = (local_30 & 0xffffff) + 0x51000000;
  local_20[1] = (local_2c & 0xffffff) + 0x50000000;
  local_20[2] = (uVar6 + 4 & 0xffff) + 0x52000000 | local_28 << 0x10;
  iVar3 = DAT_0000afdc;
  if ((uVar4 & 0xf) == 4) {
    if (((int)(uVar5 << 0x1b) < 0) && (local_24 == 0x84)) {
      uVar4 = uVar4 | 0x800;
    }
    local_20[3] = uVar4 + 0x31000000;
    local_20[4] = (uVar4 >> 8) + 0x47000000;
    local_20[5] = ((uint)*(byte *)((int)param_2 + 0x69) + iVar3 & 0x7fffff) + 0x20800000;
    uVar4 = (*param_2 & 0xf6ffffff) + 2;
    local_20[6] = (DAT_0000afe0 & uVar4) + 0x40000000;
    puVar2 = (uint *)(local_20 + 7);
    uVar4 = uVar4 & 3 | (uVar6 - 2 & 0xfff) << 0xc;
  }
  else {
    if ((int)(uVar5 << 0x1b) < 0) {
      uVar4 = uVar4 | 0x800;
    }
    local_20[3] = uVar4 + 0x31000000;
    local_20[4] = (uVar4 >> 8) + 0x47000000;
    local_20[5] = ((uint)*(byte *)((int)param_2 + 0x69) + iVar3 & 0x7fffff) + 0x20800000;
    local_20[6] = (local_18 & 0xffffff) + 0x32000000;
    local_20[7] = (*param_2 + 4 & 0x7fffff) + 0x29000000;
    if ((uVar5 & 1) == 0) {
      iVar3 = ((uint)*(byte *)((int)param_2 + 0x6a) * 2 + DAT_0000afe4 + DAT_0000afe8 & 0x7fffff) +
              0x21000000;
    }
    else {
      iVar3 = *(ushort *)(*param_2 + 0x16) + 0x32000000;
    }
    local_20[8] = iVar3;
    puVar2 = (uint *)(local_20 + 9);
    if (uVar6 - 0x18 == 0) goto LAB_0000af62;
    uVar4 = (uVar6 - 0x18 & 0xfff) << 0xc | *param_2 + 0x18 & 3;
    *puVar2 = (DAT_0000afe0 & *param_2 + 0x18 & 0xf6ffffff) + 0x40000000;
    puVar2 = (uint *)(local_20 + 10);
  }
  *puVar2 = uVar4;
  puVar2 = puVar2 + 1;
LAB_0000af62:
  *puVar2 = DAT_0000afec;
  puVar2[1] = 0xf0000000;
  return;
}



/* ======================================================================
 * 0000af6e  txp_desc_emit
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000af78) */
/* WARNING: Removing unreachable block (ram,0x0000af78) */

void txp_desc_emit(uint param_1)

{
                    /* WARNING: Could not recover jumptable at 0x0000af78. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0000af7c <= param_1) {
    param_1 = (uint)DAT_0000af7c;
  }
  (*(code *)((uint)*(byte *)(param_1 + 0xaf7d) * 2 + 0xaf7d))(0x65000000);
  return;
}



/* ======================================================================
 * 0000aff8  wsm_status_from_internal
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000affc) */
/* WARNING: Removing unreachable block (ram,0x0000affc) */

void wsm_status_from_internal(uint param_1)

{
                    /* WARNING: Could not recover jumptable at 0x0000affc. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0000b000 <= param_1) {
    param_1 = (uint)DAT_0000b000;
  }
  (*(code *)((uint)*(byte *)(param_1 + 0xb001) * 2 + 0xb001))();
  return;
}



/* ======================================================================
 * 0000b054  rx_indication_build_and_send
 * ====================================================================== */

void rx_indication_build_and_send(int param_1)

{
  ushort uVar1;
  undefined4 uVar2;
  undefined1 uVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  short *psVar7;
  uint uVar8;
  int iVar9;
  bool bVar10;
  undefined8 uVar11;
  uint local_58;
  undefined4 local_54;
  uint local_50;
  uint local_4c;
  uint local_48;
  uint local_44;
  uint local_40;
  ushort *local_3c;
  undefined4 local_38;
  int local_34;
  int local_30;
  int local_2c;
  undefined4 local_28;
  int local_24;
  int local_20;
  uint local_1c;
  undefined1 *local_18;
  
  local_2c = 0;
  local_30 = 0;
  local_34 = 0;
  local_38 = *(undefined4 *)(param_1 + 0x14);
  local_3c = *(ushort **)(param_1 + 0x1c);
  local_40 = (uint)*(ushort *)(param_1 + 0x26);
  uVar1 = *local_3c;
  uVar8 = (uint)uVar1;
  local_44 = (int)local_3c + *(int *)(param_1 + 0x18) + 7 & 0xfffffffc;
  local_18 = (undefined1 *)(param_1 + 0x20);
  local_48 = (uint)*(byte *)(param_1 + 0x2a);
  local_50 = *(uint *)(local_48 * 0x3b0 + DAT_0000b480 + 0x1c);
  local_4c = local_48;
  if (1 < local_48) {
    local_4c = 0;
  }
  iVar9 = local_4c * 0x3b0 + DAT_0000b480;
  if (*(int *)(param_1 + 0x10) << 0x18 < 0) {
    phy_restore_cfg_if_mode2(*(undefined4 *)(local_44 + 8));
  }
  uVar2 = DAT_0000b488;
  *(int *)(DAT_0000b484 + 0x3c) = *(int *)(DAT_0000b484 + 0x3c) + 1;
  *(undefined2 *)(iVar9 + 0x380) = *(undefined2 *)(param_1 + 0x28);
  *(undefined1 *)(iVar9 + 0x382) = *(undefined1 *)(param_1 + 0xf);
  *(undefined1 *)(iVar9 + 899) = *local_18;
  local_28 = *(undefined4 *)(param_1 + 0x18);
  iVar6 = *(int *)(param_1 + 0x1c);
  psVar7 = (short *)(iVar6 + -0x10);
  *(ushort *)(iVar6 + -0xe) = (ushort)(local_48 << 6) | (ushort)uVar2;
  *psVar7 = (short)local_28 + 0x10;
  uVar11 = wsm_status_from_internal(*(undefined4 *)(param_1 + 8));
  *(int *)(iVar6 + -0xc) = (int)uVar11;
  *(undefined2 *)(iVar6 + -8) = *(undefined2 *)(param_1 + 0xc);
  *(undefined1 *)(iVar6 + -6) = *(undefined1 *)(param_1 + 0xe);
  if ((~*(byte *)((int)((ulonglong)uVar11 >> 0x20) + 0xc) & 3) == 0) {
    uVar3 = *local_18;
  }
  else {
    uVar3 = *(undefined1 *)(param_1 + 0xf);
  }
  *(undefined1 *)(iVar6 + -5) = uVar3;
  iVar4 = *(int *)(param_1 + 0x10);
  *(int *)(iVar6 + -4) = iVar4;
  if (-1 < iVar4 << 0x19) {
    txbuf_freelist_push(param_1);
  }
  if (((uVar1 & 0xf) == 0) && (((uVar8 & 0xff) == 0x50 || ((uVar8 & 0xff) == 0x80)))) {
    if ((int)*(uint *)(iVar6 + -4) < 0) {
      *(uint *)(iVar6 + -4) = *(uint *)(iVar6 + -4) & 0x7fffffff;
      ind_080D_tx_trace(psVar7);
      *(uint *)(iVar6 + -4) = *(uint *)(iVar6 + -4) | 0x80000000;
    }
    else {
      ind_080D_tx_trace(psVar7);
    }
  }
  if ((((~*(uint *)(iVar6 + -4) & 0x880) == 0) &&
      (*(short *)(iVar9 + 0x42) == *(short *)(iVar6 + -8))) &&
     (*(char *)(DAT_0000b48c + 0x13) == '\0')) {
    vif_reset_state(local_4c,1);
  }
  if (((int)*(uint *)(iVar6 + -4) < 0) &&
     (((*(uint *)(iVar6 + -4) = *(uint *)(iVar6 + -4) & 0x7fffffff,
       *(int *)(local_4c * 0xc + DAT_0000b484 + 0x358) != 0 &&
       (*(uint *)(local_44 + 8) >> 0x1d == 5)) ||
      ((-1 < (int)(local_50 << 0x1d) &&
       (iVar4 = beacon_filter_check_and_store(local_4c,local_3c,local_28), iVar4 == 0)))))) {
    rxfifo_release_slot(local_38);
    return;
  }
  if (((uVar8 & 0x3ff) >> 8 == 3) && ((local_50 & 1) != 0)) goto LAB_0000b3a4;
  if (((*(uint *)(iVar9 + 0x264) & 1) != 0) &&
     (((*(int *)(iVar6 + -0xc) == 0 && ((int)(uVar8 << 0x1c) < 0)) &&
      (-1 < *(int *)(iVar6 + -4) << 0x19)))) {
    local_58 = 0;
    local_54 = 3;
    do {
      iVar4 = wsm_arg_validate_dispatch(local_58,param_1,&local_54);
      local_58 = local_58 + 1;
      if (iVar4 != 0) break;
    } while (local_58 < 0xb);
    local_34 = iVar4;
    if ((char)local_54 == '\0') {
      local_2c = 1;
      local_30 = 1;
    }
  }
  local_20 = DAT_0000b490;
  local_1c = uVar8 & 0xff;
  local_24 = DAT_0000b494;
  if ((local_1c == 0x48) || (local_1c == 200)) {
    local_2c = 0;
LAB_0000b2c2:
    if ((*(int *)(iVar9 + 0x1c) << 0x1d < 0) &&
       ((((int)(uVar8 << 0x1c) < 0 && ((local_3c[8] & 1) != 0)) &&
        ((*(byte *)(DAT_0000b490 + 4) & 1) != 0)))) {
      *(ushort *)(DAT_0000b494 + 4) = *(ushort *)(DAT_0000b494 + 4) | 2;
    }
    if (local_2c != 0) goto LAB_0000b3a4;
  }
  else {
    if ((local_2c == 0) || (local_30 == 0)) goto LAB_0000b2c2;
    bVar10 = *(int *)(iVar9 + 0x1c) << 0x1d < 0;
    do {
      if (!bVar10) goto LAB_0000b3a4;
      bVar10 = (int)(uVar8 << 0x1c) < 0;
    } while (!bVar10);
    bVar10 = (local_3c[8] & 1) == 0;
    do {
      if (bVar10) goto LAB_0000b3a4;
      bVar10 = true;
    } while ((*(byte *)(DAT_0000b490 + 4) & 1) == 0);
    if (((-1 < (int)((uint)*(ushort *)(DAT_0000b494 + 4) << 0x1e)) &&
        ((*(ushort *)(DAT_0000b494 + 4) & 1) != 0)) &&
       (uVar5 = measure_next_pending_slot(), uVar5 < 0x1e)) {
      rx_build_indication(*(undefined4 *)(uVar5 * 4 + DAT_0000b480 + 0x4b8c),param_1,uVar5);
      bVar10 = *(short *)(iVar9 + 0x15c) != 0;
      if (!bVar10) {
        evt_flags_set(DAT_0000b498,0x100);
      }
      *(bool *)(local_24 + 0x14) = bVar10;
      goto LAB_0000b3a4;
    }
    if ((*(byte *)(local_20 + 5) & 1) == 0) goto LAB_0000b3a4;
    *(ushort *)(local_24 + 4) = *(ushort *)(local_24 + 4) | 2;
  }
  if (((*(char *)(iVar9 + 0x33) == '\0') || ((*(uint *)(iVar9 + 0x1c) & 1) == 0)) ||
     (((char)local_34 != '\0' || (((local_3c[2] & 1) == 0 || (-1 < (int)(uVar8 << 0x1c))))))) {
    if ((*(ushort *)(DAT_0000b49c + 0x10) & uVar1) == *(ushort *)(DAT_0000b49c + 0x12)) {
      *(ushort *)(iVar6 + -0xe) = *(ushort *)(iVar6 + -0xe) | 0x2000;
    }
    if ((uVar8 & 0x8f) == 0x88) {
      iVar9 = (local_40 & 7) * 4 + DAT_0000b4a0;
      *(int *)(iVar9 + 300) = *(int *)(iVar9 + 300) + 1;
    }
    if (*(uint *)(DAT_0000b484 + 0x40) < 0x18) {
      uVar5 = *(uint *)(DAT_0000b484 + 0x40) + 1;
      *(uint *)(DAT_0000b484 + 0x40) = uVar5;
      if (0x17 < uVar5) {
        *DAT_0000b4a4 = *DAT_0000b4a4 | 8;
      }
      if ((local_48 < 2) && ((uVar8 & 0x7f) == 8)) {
        *(short *)(DAT_0000b4a8 + local_48 * 2) = *(short *)(DAT_0000b4a8 + local_48 * 2) + 1;
      }
      if (local_1c == 0x80) {
        *(int *)(DAT_0000b4ac + 0x18) = *(int *)(DAT_0000b4ac + 0x18) + 1;
      }
      hif_send_msg_to_host(psVar7);
      if (*(char *)(DAT_0000b494 + -0x9b) == '\0') {
        return;
      }
      if (-1 < *DAT_0000b4b0 << 0x1a) {
        return;
      }
      *(int *)(DAT_0000b494 + -0x98) = *(int *)(DAT_0000b494 + -0x98) + 1;
      return;
    }
  }
LAB_0000b3a4:
  if ((uVar8 & 0x8f) == 0x88) {
    iVar9 = (local_40 & 7) * 4 + DAT_0000b4a0;
    *(int *)(iVar9 + 0x14c) = *(int *)(iVar9 + 0x14c) + 1;
  }
  if (*(int *)(iVar6 + -4) << 0x19 < 0) {
    enc_ctx_free_offset(param_1);
  }
  rxfifo_release_slot(local_38);
  *(int *)(DAT_0000b484 + 0x30) = *(int *)(DAT_0000b484 + 0x30) + 1;
  return;
}



/* ======================================================================
 * 0000b3dc  tx_confirm_build_and_send
 * ====================================================================== */

/* tx_confirm_build_and_send(txbuf, ?) -- build the WSM TX confirm and update every
   TX statistic the host can read.  This is where throughput telemetry originates.
   
   *** MULTI-TX-CONFIRM COALESCING, with the wire layout. ***
   State machine in the HIF block at DAT_0000B490:
   
     -0x10 == 0 : no confirm pending.  If the frame's flags at +0x26 bit 5 allow
                  coalescing, start a multi confirm: MsgId := DAT_0000B5FC
                  (0x041E MULTI_TX_CONFIRM), MsgLen := 0x28, count := 1, and the
                  first entry begins at msg+4.  Otherwise emit a plain 0x0404.
     -0x10 == 1 : append.  count++ and MsgLen += 0x20, so **each entry is exactly
                  0x20 = 32 bytes** and the header carries a u32 count at +4.
     -0x10 == 2 : fw_assert(wsmlmac.c, ..., 0xC) -- overflow of the coalesce buffer.
   
   Per-confirm entry fields (offsets within the entry):
     +0x00 packet_id     +0x04 status (wsm_status_from_internal)
     +0x0C ack_failures  +0x0D retry count   +0x0E flags
     +0x10.. media delay / tx queue delay words
   
   *** THE STATISTICS COUNTERS -- all per-vif at vif*0x3B0, plus per-TID. ***
     vif+0x78  frames confirmed OK          vif+0x7C  frames confirmed failed
     vif+0x80  frames that needed >=1 retry vif+0x84  frames that needed >1 retry
     vif+0x8C  total retry count (accumulated ack_failures)
     vif+0x38  total TX confirms seen
   
     per-TID (QoS data only, tid from the QoS control field), at DAT_0000B5F8:
       +tid*4+0x2C  failed        +tid*4+0x8C  succeeded
       +tid*4+0xAC  retried once  +tid*4+0xCC  retried more than once
       +tid*4+0x10C total retries
   
   These are the numbers behind MIB 0x100A STATISTICS_TABLE and the AMPDU/TX-pipe
   counter tables, so they are the right things to sample before and after a slow
   run.  Note the per-TID set exists ONLY for QoS data frames -- the
   `(fc & 0x8F) == 0x88` test -- so non-QoS traffic contributes to the per-vif
   totals only.
   
   The coalescing here is the same mechanism hif_confirm_coalesce_hold (0x0000EC30)
   gates with its 8000-tick timer; that timer decides *when* the accumulated
   multi-confirm is flushed, this function decides *what* goes into it. */

void tx_confirm_build_and_send(int *param_1,undefined4 param_2)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  int iVar4;
  undefined4 uVar5;
  uint uVar6;
  int iVar7;
  undefined2 *puVar8;
  undefined2 *puVar9;
  int iVar10;
  bool bVar11;
  uint local_18;
  
  uVar2 = DAT_0000b4b4;
  local_18 = 0;
  puVar8 = (undefined2 *)*param_1;
  uVar6 = ((ushort)puVar8[1] & 0xff) >> 6;
  uVar3 = 0;
  if (uVar6 < 3) {
    uVar3 = uVar6;
  }
  iVar10 = uVar3 * 0x3b0 + DAT_0000b480;
  *(int *)(DAT_0000b484 + 0x38) = *(int *)(DAT_0000b484 + 0x38) + 1;
  iVar1 = DAT_0000b490;
  uVar3 = *(uint *)(*param_1 + 0xc);
  if (uVar3 >> 8 == uVar2) {
    lmc_req_confirm_and_release(uVar3 & 0xff,*param_1,param_1[8],uVar2,param_2);
    tx_wsm_buf_free(param_1);
    return;
  }
  puVar9 = puVar8;
  if (*(int *)(DAT_0000b490 + -0x14) == 0) {
LAB_0000b44a:
    if (*(int *)(iVar1 + -0x10) != 0) goto LAB_0000b454;
  }
  else {
    iVar4 = *(int *)(DAT_0000b490 + -0x10);
    if (iVar4 != 0) {
      if (iVar4 == 1) {
        hif_queue_msg_to_host(puVar8);
        *(int *)(*(int *)(iVar1 + -0xc) + 4) = *(int *)(*(int *)(iVar1 + -0xc) + 4) + 1;
        **(short **)(iVar1 + -0xc) = **(short **)(iVar1 + -0xc) + 0x20;
        puVar9 = (undefined2 *)(*(int *)(iVar1 + -8) + 0x20);
        *(undefined2 **)(iVar1 + -8) = puVar9;
        if (-1 < (int)((uint)*(ushort *)((int)param_1 + 0x26) << 0x1a)) {
          uVar5 = 2;
          goto LAB_0000b5b8;
        }
      }
      else {
        if (iVar4 != 2) goto LAB_0000b454;
        fw_assert(s_wsmlmac_c_0000b4bc,DAT_0000b4b8,0xc);
      }
      goto LAB_0000b44a;
    }
    if ((int)((uint)*(ushort *)((int)param_1 + 0x26) << 0x1a) < 0) {
      *(undefined2 **)(DAT_0000b490 + -0xc) = puVar8;
      uVar5 = DAT_0000b5fc;
      puVar9 = puVar8 + 2;
      *(undefined2 **)(iVar1 + -8) = puVar9;
      puVar8[1] = (short)uVar5;
      **(undefined2 **)(iVar1 + -0xc) = 0x28;
      uVar5 = 1;
      *(undefined4 *)(*(int *)(iVar1 + -0xc) + 4) = 1;
LAB_0000b5b8:
      *(undefined4 *)(iVar1 + -0x10) = uVar5;
      goto LAB_0000b454;
    }
  }
  *puVar9 = 0x24;
LAB_0000b454:
  bVar11 = (*(ushort *)param_1[7] & 0x8f) == 0x88;
  if (bVar11) {
    local_18 = ((ushort *)param_1[7])[0xc] & 7;
  }
  *(int *)(puVar9 + 2) = param_1[2];
  uVar5 = wsm_status_from_internal(param_1[8]);
  *(undefined4 *)(puVar9 + 4) = uVar5;
  *(char *)(puVar9 + 6) = (char)param_1[9];
  *(undefined1 *)((int)puVar9 + 0xd) = *(undefined1 *)((int)param_1 + 0x25);
  puVar9[7] = *(undefined2 *)((int)param_1 + 0x26);
  *(int *)(puVar9 + 8) = param_1[10];
  *(int *)(puVar9 + 10) = param_1[0xb];
  *(int *)(puVar9 + 0xc) = param_1[0xc];
  *(int *)(puVar9 + 0xe) = param_1[0xe];
  *(int *)(puVar9 + 0x10) = param_1[0xf];
  param_1[10] = 0;
  param_1[0xb] = 0;
  param_1[0xc] = 0;
  iVar4 = DAT_0000b5f8;
  if (*(char *)((int)puVar9 + 0xd) != '\0') {
    *(int *)(iVar10 + 0x80) = *(int *)(iVar10 + 0x80) + 1;
    *(uint *)(iVar10 + 0x8c) = *(int *)(iVar10 + 0x8c) + (uint)*(byte *)((int)puVar9 + 0xd);
    if (bVar11) {
      iVar7 = local_18 * 4 + iVar4;
      *(int *)(iVar7 + 0xac) = *(int *)(iVar7 + 0xac) + 1;
      *(uint *)(iVar7 + 0x10c) = *(int *)(iVar7 + 0x10c) + (uint)*(byte *)((int)puVar9 + 0xd);
    }
    if ((1 < *(byte *)((int)puVar9 + 0xd)) &&
       (*(int *)(iVar10 + 0x84) = *(int *)(iVar10 + 0x84) + 1, bVar11)) {
      iVar7 = local_18 * 4 + iVar4;
      *(int *)(iVar7 + 0xcc) = *(int *)(iVar7 + 0xcc) + 1;
    }
  }
  if (*(int *)(puVar9 + 4) == 0) {
    *(int *)(iVar10 + 0x78) = *(int *)(iVar10 + 0x78) + 1;
    if (bVar11) {
      iVar4 = local_18 * 4 + iVar4;
      *(int *)(iVar4 + 0x8c) = *(int *)(iVar4 + 0x8c) + 1;
    }
  }
  else {
    *(int *)(iVar10 + 0x7c) = *(int *)(iVar10 + 0x7c) + 1;
    if (bVar11) {
      iVar4 = local_18 * 4 + iVar4;
      *(int *)(iVar4 + 0x2c) = *(int *)(iVar4 + 0x2c) + 1;
    }
  }
  tx_wsm_buf_free(param_1);
  if (*(int *)(iVar1 + -0x10) == 2) {
    hif_send_msg_to_host(*(undefined4 *)(iVar1 + -0xc));
    *(undefined4 *)(iVar1 + -0x10) = 0;
  }
  else if (*(int *)(iVar1 + -0x10) == 0) {
    hif_send_msg_to_host(puVar9);
    return;
  }
  return;
}



/* ======================================================================
 * 0000b600  wsm_h_04_tx_req
 * ====================================================================== */

/* wsm_h_04_tx_req(msg) -- WSM 0x0004 TX_REQ handler.  The host bulk-data TX entry.
   
     g_stats[0x34]++;                        /* TX request counter */
     ctx = tx_wsm_buf_alloc();
     if (!ctx) { fw_assert("tx_wsm_req_01.c", 142, 1); return; }   /* <== */
     ... copy wsm_tx fields into ctx ...
     ctx->pHdr80211 = msg + 0x18 (or + 0x1A when the QoS flag in wsm_tx+0x0B is set)
     ctx->dwPayloadLen = MsgLen - 0x18 (or - 0x1A)
     tx_lmac_req_submit(ctx);
   
   *** DRIVER HAZARD: TX POOL EXHAUSTION IS A FIRMWARE ASSERT, NOT A DROP. ***
   If the host issues a TX_REQ while the firmware's WSM TX descriptor free list
   (head 0x040087B0) is empty, the firmware ASSERTS -- tx_wsm_req_01.c:142,
   reason code 1, the only assert in that translation unit (site 0x0000B61C).
   The host sees "[BH] Fatal error" / a firmware exception indication, not a
   dropped frame or an error status.
   
   The host's 30-buffer credit accounting from the startup indication is the ONLY
   thing preventing this.  A driver bug that leaks or over-issues credits does not
   degrade gracefully -- it crashes the firmware.  This makes credit accounting a
   correctness requirement rather than a flow-control optimisation.
   
   Contrast tx_ctx_alloc_init (0x0000D08C), the *internal* management-frame pool
   (3 deep, head 0x04009080): that one returns 0 and every caller drops silently.
   Two pools, both handing out 0x170-byte xr_tx_ctx structures, opposite failure
   behaviour.  In-flight count for this pool is the byte at 0x04003E9E,
   incremented by tx_wsm_buf_alloc and decremented by tx_wsm_buf_free (0x0000B702). */

void wsm_h_04_tx_req(ushort *param_1)

{
  byte bVar1;
  ushort uVar2;
  undefined4 *puVar3;
  
  *(int *)(DAT_0000b664 + 0x34) = *(int *)(DAT_0000b664 + 0x34) + 1;
  puVar3 = (undefined4 *)tx_wsm_buf_alloc();
  if (puVar3 == (undefined4 *)0x0) {
    fw_assert(s_tx_wsm_req_01_c_0000b668,0x8e,1);
    return;
  }
  puVar3[2] = *(undefined4 *)(param_1 + 2);
  *(char *)(puVar3 + 3) = (char)param_1[4];
  *(undefined1 *)((int)puVar3 + 0xd) = *(undefined1 *)((int)param_1 + 9);
  *(char *)((int)puVar3 + 0xe) = (char)param_1[5];
  puVar3[4] = *(undefined4 *)(param_1 + 8);
  puVar3[5] = *(undefined4 *)(param_1 + 10);
  *(char *)(puVar3 + 9) = (char)param_1[4];
  bVar1 = *(byte *)((int)param_1 + 0xb);
  *(byte *)((int)puVar3 + 0xf) = bVar1;
  puVar3[7] = param_1 + 0xc;
  uVar2 = *param_1;
  puVar3[6] = uVar2 - 0x18;
  if ((int)((uint)bVar1 << 0x18) < 0) {
    puVar3[7] = param_1 + 0xd;
    puVar3[6] = uVar2 - 0x1a;
  }
  *puVar3 = param_1;
  tx_lmac_req_submit();
  return;
}



/* ======================================================================
 * 0000b678  txbuf_freelist_pop
 * ====================================================================== */

void txbuf_freelist_pop(void)

{
  int *piVar1;
  byte bVar2;
  
  piVar1 = *(int **)(DAT_0000b738 + 0x14);
  if (piVar1 != (int *)0x0) {
    *(int *)(DAT_0000b738 + 0x14) = piVar1[1];
    *(undefined4 *)(*piVar1 + 4) = 0;
    bVar2 = *(char *)(DAT_0000b73c + 0x1b) + 1;
    *(byte *)(DAT_0000b73c + 0x1b) = bVar2;
    if (0xf < bVar2) {
      *DAT_0000b740 = *DAT_0000b740 | 4;
    }
  }
  return;
}



/* ======================================================================
 * 0000b6a6  txbuf_freelist_push
 * ====================================================================== */

void txbuf_freelist_push(int param_1)

{
  int iVar1;
  uint *puVar2;
  uint uVar3;
  
  iVar1 = DAT_0000b738;
  *(undefined4 *)(param_1 + 4) = *(undefined4 *)(DAT_0000b738 + 0x14);
  *(int *)(iVar1 + 0x14) = param_1;
  puVar2 = DAT_0000b740;
  *(char *)(DAT_0000b73c + 0x1b) = *(char *)(DAT_0000b73c + 0x1b) + -1;
  uVar3 = *puVar2;
  if ((int)(uVar3 << 0x1d) < 0) {
    *puVar2 = uVar3 & 0xfffffffb;
    evt_flags_set(DAT_0000b744,0x80000);
  }
  return;
}



/* ======================================================================
 * 0000b6d0  tx_wsm_buf_alloc
 * ====================================================================== */

int tx_wsm_buf_alloc(void)

{
  int iVar1;
  int iVar2;
  
  irq_disable_save();
  iVar2 = *(int *)(DAT_0000b738 + 0x18);
  if (iVar2 != 0) {
    *(undefined4 *)(DAT_0000b738 + 0x18) = *(undefined4 *)(iVar2 + 4);
    *(undefined1 *)(iVar2 + 0xf) = 0;
    *(undefined4 *)(iVar2 + 0x20) = 0xfe;
    *(undefined2 *)(iVar2 + 0x70) = 0xfe;
    iVar1 = DAT_0000b748;
    *(undefined4 *)(iVar2 + 0x80) = 0;
    *(char *)(iVar1 + 6) = *(char *)(iVar1 + 6) + '\x01';
  }
  irq_restore();
  return iVar2;
}



/* ======================================================================
 * 0000b702  tx_wsm_buf_free
 * ====================================================================== */

void tx_wsm_buf_free(int param_1)

{
  int iVar1;
  int iVar2;
  
  irq_disable_save();
  *(undefined4 *)(param_1 + 0x20) = 0xff;
  *(undefined2 *)(param_1 + 0x70) = 0xff;
  *(uint *)(param_1 + 0x80) = *(uint *)(param_1 + 0x80) | 0x40000;
  iVar1 = DAT_0000b738;
  *(undefined4 *)(param_1 + 4) = *(undefined4 *)(DAT_0000b738 + 0x18);
  iVar2 = DAT_0000b748;
  *(int *)(iVar1 + 0x18) = param_1;
  *(char *)(iVar2 + 6) = *(char *)(iVar2 + 6) + -1;
  irq_restore();
  return;
}



/* ======================================================================
 * 0000b74c  tx_frame_complete
 * ====================================================================== */

/* tx_frame_complete(tx_ctx, status) -- TX completion for one frame.
   Idempotent: does nothing if ctx[0x70] is already 0xFF.
   
     ctx[0x25] = ctx[0x72];  ctx[0x70] = status;  ctx[0x20] = status
     decrement the pending count at *(ctx[0x4C] + 7)
     vif[0x3C4] |= 1 when status == 0 and (ctx[0x5E] & 0xF) == 8
     dispatch: (*(fn_ptr *)(0x???? + ctx[0x53]*4))(ctx)     -- per-class
               completion callback table, indexed by the class byte that
               tx_ctx_alloc_init stored at +0x53
     evt_flags_set(..., 0x200000)
     then tx_ctx_free (0x0000D100)
   
   Link-quality counter: on success (status == 0) it increments vif[0x27],
   saturating at 5, then subtracts ctx[0x25] from it (flooring at 0).
   lmc_tx_assign_default_rate (0x0000456C) reads `vif[0x27] < 3` to decide
   whether to take the default TX rate from vif[0x12C]/vif[0x12D] or fall back
   to vif[0x24].  So vif[0x27] is a confidence counter gating firmware-chosen
   rates for internally generated frames -- distinct from the host's rate
   policy, which only covers host-submitted data frames. */

void tx_frame_complete(int param_1,undefined4 param_2)

{
  int iVar1;
  int *piVar2;
  byte bVar3;
  char cVar4;
  uint uVar5;
  
  if (*(short *)(param_1 + 0x70) != 0xff) {
    *(char *)(param_1 + 0x25) = (char)*(undefined2 *)(param_1 + 0x72);
    *(short *)(param_1 + 0x70) = (short)param_2;
    *(undefined4 *)(param_1 + 0x20) = param_2;
    iVar1 = *(int *)(param_1 + 0x4c);
    if (((iVar1 != 0) && ('\0' < *(char *)(iVar1 + 7))) && (*(char *)(param_1 + 0x53) == '\0')) {
      *(char *)(iVar1 + 7) = *(char *)(iVar1 + 7) + -1;
    }
    if (*(byte *)(param_1 + 0xbd) < 2) {
      iVar1 = (uint)*(byte *)(param_1 + 0xbd) * 0x3b0 + DAT_0000bb58;
      bVar3 = 0;
      if ((*(int *)(param_1 + 0x20) == 0) && ((*(ushort *)(param_1 + 0x5e) & 0xf) == 8)) {
        bVar3 = 1;
      }
      *(byte *)(iVar1 + 0x3c4) = bVar3 | *(byte *)(iVar1 + 0x3c4);
      uVar5 = *(uint *)(iVar1 + 0x1c);
      if ((((uVar5 & 7) >> 1 == 3) && (cVar4 = *(char *)(iVar1 + 0x163), cVar4 != '\0')) &&
         (((*(byte *)(*(int *)(param_1 + 0x54) + 4) & 1) != 0 &&
          (*(char *)(iVar1 + 0x163) = cVar4 + -1, cVar4 == '\x01')))) {
        if ((int)(uVar5 << 1) < 0) {
          *(uint *)(iVar1 + 0x1c) = uVar5 | 0x20000000;
        }
        *(uint *)(iVar1 + 0x1c) = *(uint *)(iVar1 + 0x1c) & 0x7fffffff;
      }
    }
    piVar2 = (int *)((uint)*(byte *)(param_1 + 0x53) * 4 + DAT_0000bb5c);
    if (*piVar2 != 0) {
      *(uint *)(param_1 + 0x80) = *(uint *)(param_1 + 0x80) | 0x10000;
      (*(code *)*piVar2)(param_1);
      evt_flags_set(DAT_0000bb60,0x200000);
      if (*(char *)(param_1 + 0x53) != '\0') {
        if (*(byte *)(param_1 + 0xbd) < 2) {
          iVar1 = (uint)*(byte *)(param_1 + 0xbd) * 0x3b0 + DAT_0000bb58;
          if (*(char *)(iVar1 + 0x26) == '\0') {
            bVar3 = *(byte *)(iVar1 + 300);
          }
          else {
            bVar3 = *(byte *)(iVar1 + 0x12d);
          }
          if ((bVar3 != 0xff) && (*(byte *)(param_1 + 99) <= bVar3)) {
            if ((*(int *)(param_1 + 0x20) == 0) && (*(byte *)(iVar1 + 0x27) < 5)) {
              *(byte *)(iVar1 + 0x27) = *(byte *)(iVar1 + 0x27) + 1;
            }
            bVar3 = *(byte *)(param_1 + 0x25);
            if (bVar3 != 0) {
              if (*(byte *)(iVar1 + 0x27) < bVar3) {
                cVar4 = '\0';
              }
              else {
                cVar4 = *(byte *)(iVar1 + 0x27) - bVar3;
              }
              *(char *)(iVar1 + 0x27) = cVar4;
            }
          }
        }
        tx_ctx_free(param_1);
      }
    }
  }
  return;
}



/* ======================================================================
 * 0000b88e  task_b88e
 * ====================================================================== */

void task_b88e(void)

{
  bool bVar1;
  bool bVar2;
  int iVar3;
  uint uVar4;
  xr_tx_ctx *ctx;
  int iVar5;
  xr_tx_ctx *pxVar6;
  undefined4 local_20;
  xr_tx_ctx *local_1c;
  undefined1 *local_18;
  
  local_20 = 0x14;
  iVar3 = fw_read_timer();
  pxVar6 = (xr_tx_ctx *)*DAT_0000bb64;
  ctx = (xr_tx_ctx *)*DAT_0000bb64;
  do {
    local_1c = ctx;
    ctx = pxVar6;
    if (ctx == (xr_tx_ctx *)0x0) {
      txp_scheduler_run();
      return;
    }
    local_18 = &ctx->field_0xa0;
    bVar1 = false;
    bVar2 = false;
    if ((*DAT_0000bb68 & 0xa0) == 0) {
      iVar5 = (uint)(byte)ctx->field_0xbd * 0x3b0 + DAT_0000bb58;
      uVar4 = 1 << (uint)(byte)ctx->field_0xbf;
      if ((*(ushort *)(iVar5 + 0x2c) & uVar4) == 0) {
        bVar1 = true;
        bVar2 = true;
        local_20 = 0x14;
      }
      else if (((*(ushort *)(iVar5 + 0x2e) & uVar4) == 0) &&
              ((*(ushort *)&ctx->field_0x5e & 0xff) != 0xd0)) {
        if (*(ushort *)(iVar5 + 0x2e) == 0) {
          if ((int)(*(uint *)(iVar5 + 0x1c) << 1) < 0) {
            *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) | 0x4000000;
          }
          else {
            vif_resume_tx_after_radio();
          }
          if (((uint)*(ushort *)(iVar5 + 0x2e) & 1 << local_18[0x1f]) != 0) goto LAB_0000b936;
        }
      }
      else if ((int)(*(uint *)(iVar5 + 0x1c) << 2) < 0) {
        *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) | 0x4000000;
      }
      else {
LAB_0000b936:
        bVar1 = true;
      }
      if (((*(char *)(iVar5 + 0x18) == '\x04') || (*(char *)(iVar5 + 0x18) == '\x06')) &&
         ((ctx->bCompletionClass == 6 ||
          (iVar5 = txp_program_pipe_hw(&ctx->field_0x54,0), iVar5 != 0)))) goto LAB_0000b8c6;
      if (!bVar1) goto LAB_0000b95c;
    }
    else {
      if ((ctx->bCompletionClass != 6) && (ctx->bCompletionClass != 9)) {
LAB_0000b95c:
        if ((-1 < (*(int *)&ctx->field_0x40 - iVar3) + DAT_0000bb6c) || (ctx->bCompletionClass != 0)
           ) goto LAB_0000b978;
        bVar2 = true;
        local_20 = 10;
      }
LAB_0000b8c6:
      bVar1 = true;
    }
LAB_0000b978:
    pxVar6 = (xr_tx_ctx *)ctx->dwNextFree;
    if (bVar1) {
      txq_list_remove(ctx,&local_1c);
      if (bVar2) {
        tx_frame_complete(ctx,local_20);
        ctx = local_1c;
      }
      else {
        tx_frame_done_release(ctx);
        ctx = local_1c;
      }
    }
  } while( true );
}



/* ======================================================================
 * 0000b9ba  tx_frame_complete_stats
 * ====================================================================== */

void tx_frame_complete_stats(int *param_1)

{
  ushort uVar1;
  byte bVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  
  uVar3 = 0;
  if (*param_1 == 0) {
    tx_wsm_buf_free();
    return;
  }
  param_1[8] = (uint)*(ushort *)(param_1 + 0x1c);
  *(undefined1 *)(param_1 + 9) = *(undefined1 *)((int)param_1 + 99);
  param_1[10] = param_1[0x1d];
  param_1[0xb] = param_1[0x1e];
  param_1[0xc] = param_1[0x1f];
  param_1[0x1d] = 0;
  param_1[0x1e] = 0;
  param_1[0x1f] = 0;
  param_1[0xe] = param_1[0x1a] - param_1[0x10];
  param_1[0xf] = param_1[0x1b] - param_1[0x10];
  if ((*(ushort *)(param_1 + 0x1c) == 0) && (*(byte *)((int)param_1 + 0xbd) < 2)) {
    iVar5 = (uint)*(byte *)((int)param_1 + 0xbd) * 0x3b0 + DAT_0000bb58;
    uVar1 = *(ushort *)(iVar5 + 0x15e) & ~(ushort)(1 << *(sbyte *)((int)param_1 + 0xbf));
    *(ushort *)(iVar5 + 0x15e) = uVar1;
    if (*(short *)(iVar5 + 0x2e) != 0) {
      *(ushort *)(iVar5 + 0x2e) =
           (*(ushort *)(iVar5 + 0x160) | ~*(ushort *)(iVar5 + 0x15c) | uVar1) &
           *(ushort *)(iVar5 + 0x2c);
    }
    if (((*(char *)(iVar5 + 0x3a1) != '\0') && (*(char *)(iVar5 + 0x3a0) != '\0')) &&
       (*(char *)((int)param_1 + 0xbf) != '\0')) {
      iVar6 = DAT_0000bb58 + DAT_0000bb70;
      if ((*(char *)(iVar5 + 0x18) == '\x04') || (*(char *)(iVar5 + 0x18) == '\x06')) {
        for (; uVar3 < *(ushort *)(iVar6 + 0x14); uVar3 = uVar3 + 1 & 0xff) {
          if (*(char *)((int)param_1 + 0xbf) ==
              *(char *)(uVar3 * 0xc + DAT_0000bb58 + DAT_0000bb70 + 0x18)) {
            iVar4 = uVar3 * 0xc + DAT_0000bb58 + DAT_0000bb70;
            *(char *)(iVar4 + 0x1b) = *(char *)(iVar5 + 0x3a0) + *(char *)(iVar5 + 0x3a1);
            bVar2 = *(byte *)(iVar4 + 0x1c) >> 1;
            *(byte *)(iVar4 + 0x1c) = bVar2 << 1;
            if ((bVar2 == 0) && (*(char *)(iVar4 + 0x1d) == '\0')) {
              *(ushort *)(iVar6 + 0x16) =
                   *(ushort *)(iVar6 + 0x16) & ~(ushort)(1 << *(sbyte *)(iVar4 + 0x18));
            }
            break;
          }
        }
      }
    }
  }
  param_1[0x20] = param_1[0x20] | 0x20000;
  tx_confirm_build_and_send();
  return;
}



/* ======================================================================
 * 0000bac6  link_lookup_by_mac
 * ====================================================================== */

undefined1 link_lookup_by_mac(undefined4 param_1,byte *param_2)

{
  uint uVar1;
  int iVar2;
  short *psVar3;
  
  if ((*param_2 & 1) != 0) {
    return 0;
  }
  uVar1 = 0;
  while( true ) {
    if (*(ushort *)(DAT_0000bb58 + DAT_0000bb70 + 0x14) <= uVar1) {
      return 0xf;
    }
    iVar2 = uVar1 * 0xc + DAT_0000bb58;
    if (((*(short *)(iVar2 + DAT_0000bb70 + 0x1e) == *(short *)param_2) &&
        (psVar3 = (short *)(iVar2 + DAT_0000bb70 + 0x20), *psVar3 == *(short *)(param_2 + 2))) &&
       (psVar3[1] == *(short *)(param_2 + 4))) break;
    uVar1 = uVar1 + 1;
  }
  return *(undefined1 *)(uVar1 * 0xc + DAT_0000bb58 + DAT_0000bb70 + 0x18);
}



/* ======================================================================
 * 0000bb1e  pipe_find_by_mac_upper
 * ====================================================================== */

short * pipe_find_by_mac_upper(short *param_1)

{
  short *psVar1;
  uint uVar2;
  
  uVar2 = 4;
  while ((((psVar1 = (short *)(uVar2 * 8 + DAT_0000bb58 + DAT_0000bb74),
           (*(byte *)(psVar1 + 3) & 1) == 0 || (*psVar1 != *param_1)) || (psVar1[1] != param_1[1]))
         || (psVar1[2] != param_1[2]))) {
    uVar2 = uVar2 + 1;
    if (7 < uVar2) {
      return (short *)0x0;
    }
  }
  return psVar1;
}



/* ======================================================================
 * 0000bb9a  vif_reset_state
 * ====================================================================== */

void vif_reset_state(int param_1,int param_2)

{
  char cVar1;
  byte bVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  int iVar6;
  char *dst;
  
  iVar3 = param_1 * 0x3b0 + DAT_0000bd64;
  iVar6 = param_1 * 0x3b0 + DAT_0000bd64;
  dst = (char *)(iVar6 + 0x388);
  if (param_2 == 0) {
    fw_memzero(dst,0x16);
    return;
  }
  if ((*(byte *)(iVar3 + 900) & 1) == 0) {
    return;
  }
  iVar4 = param_1 * 0x3b0 + DAT_0000bd64 + 0x38e;
  if (-1 < (int)((uint)*(byte *)(iVar3 + 900) * 0x40000000)) {
    *(undefined1 *)((uint)*(byte *)(iVar6 + 0x38a) + iVar4) = *(undefined1 *)(iVar3 + 0x382);
    *(ushort *)(iVar6 + 0x38c) = *(short *)(iVar6 + 0x38c) + (ushort)*(byte *)(iVar3 + 0x382);
    *(byte *)(iVar6 + 0x38a) = *(char *)(iVar6 + 0x38a) + 1U & 0xf;
    if (*(char *)(iVar6 + 0x38b) == *(char *)(iVar3 + 0x387)) {
      *(ushort *)(iVar6 + 0x38c) =
           *(short *)(iVar6 + 0x38c) - (ushort)*(byte *)((uint)*(byte *)(iVar6 + 0x389) + iVar4);
      *(byte *)(iVar6 + 0x389) = *(byte *)(iVar6 + 0x389) + 1 & 0xf;
    }
    if (*(byte *)(iVar6 + 0x38b) < *(byte *)(iVar3 + 0x387)) {
      *(byte *)(iVar6 + 0x38b) = *(byte *)(iVar6 + 0x38b) + 1;
    }
    if (*(char *)(iVar6 + 0x38b) != *(char *)(iVar3 + 0x387)) {
      return;
    }
    bVar2 = __udivsi3(*(undefined2 *)(iVar6 + 0x38c));
    *(byte *)(DAT_0000bd68 + 6) = bVar2;
    uVar5 = (uint)*(byte *)(iVar3 + 900);
    if ((uVar5 & 0xf) >> 2 == 0) {
      cVar1 = *dst;
      if (cVar1 == '\0') {
        if (bVar2 < *(byte *)(iVar3 + 0x385)) goto LAB_0000bd18;
      }
      else if (cVar1 != '\x01') {
        if (cVar1 != '\x02') {
          return;
        }
        if (*(byte *)(iVar3 + 0x385) <= bVar2) {
          return;
        }
LAB_0000bd18:
        *dst = '\x01';
        event_send_rcpi_rssi(param_1);
        syn_scan_try_start();
        return;
      }
      if (bVar2 <= *(byte *)(iVar3 + 0x386)) {
        return;
      }
      *dst = '\x02';
    }
    else {
      if (-1 < (int)(uVar5 << 0x1d)) {
        return;
      }
      if ((int)(uVar5 << 0x1c) < 0) {
        return;
      }
      if (*dst != '\0') {
        if (*dst != '\x01') {
          return;
        }
        if (bVar2 < *(byte *)(iVar3 + 0x385)) {
          return;
        }
        goto LAB_0000bd4a;
      }
      if (*(byte *)(iVar3 + 0x385) <= bVar2) {
        return;
      }
      *dst = '\x01';
    }
    goto LAB_0000bd58;
  }
  *(undefined1 *)((uint)*(byte *)(iVar6 + 0x38a) + iVar4) = *(undefined1 *)(iVar3 + 899);
  *(short *)(iVar6 + 0x38c) = *(short *)(iVar6 + 0x38c) + (short)*(char *)(iVar3 + 899);
  *(byte *)(iVar6 + 0x38a) = *(char *)(iVar6 + 0x38a) + 1U & 0xf;
  if (*(char *)(iVar6 + 0x38b) == *(char *)(iVar3 + 0x387)) {
    *(short *)(iVar6 + 0x38c) =
         *(short *)(iVar6 + 0x38c) - (short)*(char *)((uint)*(byte *)(iVar6 + 0x389) + iVar4);
    *(byte *)(iVar6 + 0x389) = *(byte *)(iVar6 + 0x389) + 1 & 0xf;
  }
  if (*(byte *)(iVar6 + 0x38b) < *(byte *)(iVar3 + 0x387)) {
    *(byte *)(iVar6 + 0x38b) = *(byte *)(iVar6 + 0x38b) + 1;
  }
  if (*(char *)(iVar6 + 0x38b) != *(char *)(iVar3 + 0x387)) {
    return;
  }
  bVar2 = fw_div_scaled((int)*(short *)(iVar6 + 0x38c));
  uVar5 = (uint)*(byte *)(iVar3 + 900);
  if ((uVar5 & 0xf) >> 2 == 0) {
    cVar1 = *dst;
    if (cVar1 == '\0') {
      if ((char)bVar2 < *(char *)(iVar3 + 0x385)) goto LAB_0000bc64;
    }
    else if (cVar1 != '\x01') {
      if (cVar1 != '\x02') {
        return;
      }
      goto LAB_0000bc7a;
    }
    if ((char)bVar2 <= *(char *)(iVar3 + 0x386)) {
      return;
    }
    *dst = '\x02';
  }
  else {
    if (-1 < (int)(uVar5 << 0x1d)) {
      return;
    }
    if ((int)(uVar5 << 0x1c) < 0) {
      return;
    }
    if (*dst != '\0') {
      if (*dst != '\x01') {
        return;
      }
      if ((char)bVar2 < *(char *)(iVar3 + 0x385)) {
        return;
      }
LAB_0000bd4a:
      *dst = '\0';
      return;
    }
LAB_0000bc7a:
    if (*(char *)(iVar3 + 0x385) <= (char)bVar2) {
      return;
    }
LAB_0000bc64:
    *dst = '\x01';
  }
LAB_0000bd58:
  event_send_rcpi_rssi(param_1,bVar2);
  return;
}



/* ======================================================================
 * 0000bd6c  rx_deliver_or_queue_mgmt
 * ====================================================================== */

void rx_deliver_or_queue_mgmt(int *param_1)

{
  int iVar1;
  
  if ((*(byte *)((int)param_1 + 0x2a) < 2) && (*(char *)((int)param_1 + 0x2b) == -0x30)) {
    if (*(char *)(param_1[7] + 0x18) == '\x03') {
      iVar1 = rx_mgmt_queue_to_host
                        (param_1[7],param_1[6] & 0xffff,*(undefined1 *)((int)param_1 + 0xe),
                         *(undefined1 *)(*param_1 + 10),*(byte *)((int)param_1 + 0x2a));
      if ((iVar1 != 0) && ((iVar1 == 2 || (iVar1 = txp_should_defer_frame(param_1), iVar1 == 1)))) {
        rxfifo_release_slot(param_1[5]);
        txbuf_freelist_push(param_1);
        return;
      }
    }
  }
  rx_indication_build_and_send(param_1);
  return;
}



/* ======================================================================
 * 0000bdc0  rx_frame_dispatch
 * ====================================================================== */

void rx_frame_dispatch(int param_1)

{
  char cVar1;
  int iVar2;
  uint uVar3;
  bool bVar4;
  int *piVar5;
  int iVar6;
  
  if ((int)(*DAT_0000c1b8 << 0x1e) < 0) {
    *DAT_0000c1b8 = *DAT_0000c1b8 & 0xfffffffd;
    evt_flags_set(DAT_0000c1bc,0x80000);
  }
  piVar5 = *(int **)(param_1 + -0x2c);
  iVar6 = (uint)*(byte *)((int)piVar5 + 0x2a) * 0x3b0 + DAT_0000c1c0;
  if (piVar5 == (int *)0x0) {
    return;
  }
  if (*(char *)(param_1 + 5) == '\0') {
    iVar2 = *piVar5;
    if ((*(char *)(iVar2 + 9) == '\x02') || (*(char *)(iVar2 + 9) == '\x03')) {
      fw_memcpy(*(void **)(iVar2 + 0x48),(void *)(iVar2 + 0xa4),4);
    }
    uVar3 = (uint)*(ushort *)piVar5[7];
    bVar4 = true;
    if (((((ushort *)piVar5[7])[0xb] & 0xf) == 0) && (-1 < (int)(uVar3 << 0x15))) {
      bVar4 = false;
    }
    if ((uVar3 & 0xf) == 4) {
      bVar4 = false;
    }
    if ((uVar3 & 0xff) == 0xa4) goto LAB_0000be3c;
    if (bVar4) {
      lmc_msg_release_slot(piVar5);
      return;
    }
    iVar2 = *piVar5;
    cVar1 = *(char *)(iVar2 + 9);
    if ((cVar1 == '\x02') || (cVar1 == '\x03')) {
      rx_reorder_or_deliver(piVar5);
      return;
    }
    if ((cVar1 != '\b') ||
       (iVar2 = fw_mem_equal(iVar2 + 0xd0,8,*(int *)(iVar2 + 0x1c) + 10,8), iVar2 != 0)) {
      rx_deliver_or_queue_mgmt(piVar5);
      return;
    }
    *(int *)(iVar6 + 0xa0) = *(int *)(iVar6 + 0xa0) + 1;
    iVar6 = 0x13;
  }
  else {
    if ((*(int *)(*piVar5 + 4) != 0) || (*(char *)(*piVar5 + 8) != '\0')) {
      return;
    }
    *(int *)(iVar6 + 0x68) = *(int *)(iVar6 + 0x68) + 1;
    if ((*(byte *)((int)piVar5 + 0x2b) & 0xf) == 0) {
      *(int *)(iVar6 + 0x94) = *(int *)(iVar6 + 0x94) + 1;
    }
    iVar6 = 0x15;
  }
  piVar5[2] = iVar6;
  piVar5[6] = *(int *)(*piVar5 + 0x10);
LAB_0000be3c:
  rx_indication_build_and_send(piVar5);
  return;
}



/* ======================================================================
 * 0000beba  rx_decrypt_and_verify
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000c058) */
/* WARNING: Removing unreachable block (ram,0x0000c058) */

undefined4
rx_decrypt_and_verify
          (int *param_1,uint param_2,int param_3,undefined4 param_4,ushort param_5,int param_6)

{
  ushort uVar1;
  byte *pbVar2;
  int *piVar3;
  int iVar4;
  undefined4 uVar5;
  int iVar6;
  uint uVar7;
  int iVar8;
  int iVar9;
  bool bVar10;
  undefined4 local_54;
  int local_50;
  undefined1 auStack_4c [4];
  int local_48;
  uint local_44;
  int local_30;
  int local_2c;
  int *local_28;
  int *piStack_24;
  uint local_20;
  int local_1c;
  undefined4 local_18;
  
  iVar9 = (uint)*(byte *)((int)param_1 + 0x17) * 0x3b0 + DAT_0000c1c0;
  if (*(byte *)((int)param_1 + 0x17) < 2) {
    uVar7 = *(uint *)(iVar9 + 0x1c);
  }
  else {
    uVar7 = 0;
  }
  local_48 = *param_1;
  piStack_24 = param_1;
  local_20 = param_2;
  local_1c = param_3;
  local_18 = param_4;
  if ((param_2 < 0x19) && (*(short *)((int)param_1 + 0x12) != 0x84)) {
    if ((uVar7 & DAT_0000c1c4) == 0) {
      return 0;
    }
    if (*(short *)((int)param_1 + 0x12) == 0xa4) {
      uVar1 = (ushort)(1 << (sbyte)param_1[6]) | *(ushort *)(iVar9 + 0x15e);
      *(ushort *)(iVar9 + 0x15e) = uVar1;
      if (*(short *)(iVar9 + 0x2e) != 0) {
        *(ushort *)(iVar9 + 0x2e) =
             (*(ushort *)(iVar9 + 0x160) | ~*(ushort *)(iVar9 + 0x15c) | uVar1) &
             *(ushort *)(iVar9 + 0x2c);
        evt_flags_set(DAT_0000c1bc,0x200000);
      }
    }
  }
  if (DAT_0000c1c8 < local_20) {
    return 0;
  }
  piVar3 = (int *)txbuf_freelist_pop();
  if (piVar3 == (int *)0x0) {
    return 0;
  }
  piVar3[5] = param_1[9];
  local_28 = piVar3 + 8;
  *(int *)(*piVar3 + 0x10) = local_1c;
  *(short *)(piVar3 + 9) = (short)local_18;
  *(ushort *)((int)piVar3 + 0x26) = param_5;
  iVar4 = *param_1;
  piVar3[7] = iVar4;
  piVar3[6] = local_20;
  *(int *)(*piVar3 + 0xc) = iVar4 + local_1c;
  *(short *)(piVar3 + 3) = (short)param_1[7];
  *(undefined1 *)((int)piVar3 + 0xe) = *(undefined1 *)((int)param_1 + 0x16);
  *(char *)local_28 = (char)param_1[5];
  *(undefined1 *)((int)piVar3 + 0xf) = *(undefined1 *)((int)param_1 + 0x15);
  *(undefined2 *)(piVar3 + 10) = *(undefined2 *)((int)param_1 + 0x1a);
  *(int *)(*piVar3 + 0x14) = param_1[2];
  *(int *)(*piVar3 + 0x18) = param_6;
  piVar3[4] = param_1[8];
  piVar3[2] = 0;
  *(char *)((int)piVar3 + 0x2b) = (char)*(undefined2 *)((int)param_1 + 0x12);
  *(undefined1 *)(*piVar3 + 10) = *(undefined1 *)((int)param_1 + 7);
  *(undefined1 *)((int)piVar3 + 0x2a) = *(undefined1 *)((int)param_1 + 0x17);
  piVar3[4] = piVar3[4] | (*(byte *)(param_1 + 6) & 0xf) << 0x19;
  local_30 = local_20 - local_1c;
  iVar4 = *piVar3;
  iVar8 = iVar4 + 0x2c;
  local_2c = local_48 + local_1c;
  *(int *)(iVar4 + 0x48) = local_2c;
  *(undefined4 *)(iVar4 + 0x50) = DAT_0000c1cc;
  local_44 = (uint)*(ushort *)(param_1 + 4);
  if (((int)(local_44 << 0x11) < 0) &&
     ((*(short *)((int)param_1 + 0x12) != 0xb0 || (-1 < *(int *)(iVar9 + 0x138) << 0x1d)))) {
    local_54 = 0;
    if (param_6 == 0) {
      *(int *)(iVar9 + 0x70) = *(int *)(iVar9 + 0x70) + 1;
      piVar3[2] = 0x10;
      if (*(char *)(iVar9 + 0x18) != '\a') {
        piVar3[6] = *(int *)(*piVar3 + 0x10);
      }
      bVar10 = *(char *)(iVar9 + 0x1a) == '\x02';
    }
    else {
      local_50 = lmc_msg_dispatch_by_type(param_6,iVar8,auStack_4c,&local_54);
      bVar10 = local_50 == 9;
      if (!bVar10) {
        if ((char)local_54 == '\x01') {
          piVar3[4] = piVar3[4] | 0x80000;
        }
        *(char *)(*piVar3 + 9) = (char)local_50;
        if ((local_44 & 0x8f) == 0x88) {
          param_5 = param_5 & 0xf;
          if (7 < param_5) {
            param_5 = 0;
          }
          qos_dispatch(param_5);
        }
                    /* WARNING: Could not recover jumptable at 0x0000c058. Too many branches */
                    /* WARNING: Treating indirect jump as call */
        if (local_50 - 2U < (uint)DAT_0000c05c) {
          pbVar2 = (byte *)(local_50 + 0xc05b);
        }
        else {
          pbVar2 = (byte *)(DAT_0000c05c + 0xc05d);
        }
        uVar5 = (*(code *)((uint)*pbVar2 * 2 + 0xc05d))(3);
        return uVar5;
      }
    }
    if (!bVar10) {
LAB_0000bffe:
      rx_indication_build_and_send(piVar3);
      return 1;
    }
  }
  else {
    bVar10 = (*(uint *)(iVar9 + 0x138) & 1) == 0;
    do {
      if (bVar10) goto LAB_0000c33a;
      iVar6 = frame_is_unprotected_mgmt(*param_1);
      bVar10 = true;
    } while (iVar6 == 0);
    if (((*(byte *)(*param_1 + 4) & 1) == 0) &&
       ((*(short *)((int)param_1 + 0x12) == 0xa0 || (*(short *)((int)param_1 + 0x12) == 0xc0)))) {
LAB_0000c33a:
      *(int *)(iVar4 + 0x4c) = local_30;
      *(undefined1 *)(*piVar3 + 9) = 9;
      *(undefined1 *)(iVar4 + 0x30) = 10;
      *(undefined1 *)(iVar4 + 0x31) = 0;
      *(undefined4 *)(iVar4 + 0x44) = *(undefined4 *)(*piVar3 + 0xc);
      rx_frame_dispatch(iVar8);
      return 1;
    }
    iVar6 = ie_find_in_mgmt_frame(*param_1,local_20,0x4c,0);
    *(int *)(*piVar3 + 0x1c) = iVar6;
    if (iVar6 != 0) {
      iVar6 = key_lookup_for_frame
                        (*(undefined1 *)((int)param_1 + 0x17),*param_1 + 10,
                         *(ushort *)(iVar6 + 2) & 0xff);
      if (iVar6 == 0) {
        *(int *)(iVar9 + 0x70) = *(int *)(iVar9 + 0x70) + 1;
        piVar3[2] = 0x10;
        piVar3[6] = *(int *)(*piVar3 + 0x10);
        goto LAB_0000bffe;
      }
      fw_memcpy((void *)(iVar4 + 0x34),(void *)(iVar6 + 4),0x10);
      *(undefined1 *)(*piVar3 + 9) = 8;
      *(undefined2 *)(*piVar3 + 0x24) = *(undefined2 *)(*(int *)(*piVar3 + 0x1c) + 4);
      *(undefined2 *)(*piVar3 + 0x26) = *(undefined2 *)(*(int *)(*piVar3 + 0x1c) + 6);
      uVar1 = *(ushort *)(*(int *)(*piVar3 + 0x1c) + 8);
      *(ushort *)(*piVar3 + 0x28) = uVar1;
      uVar7 = *(uint *)(*piVar3 + 0x24);
      if (((*(uint *)(iVar6 + 0x2c) < uVar7) ||
          ((uVar7 == *(uint *)(iVar6 + 0x2c) && (*(ushort *)(iVar6 + 0x30) < uVar1)))) &&
         ((uVar1 != 0 || (uVar7 != 0)))) {
        *(undefined2 *)(iVar6 + 0x2c) = *(undefined2 *)(*piVar3 + 0x24);
        *(undefined2 *)(iVar6 + 0x2e) = *(undefined2 *)(*piVar3 + 0x26);
        *(undefined2 *)(iVar6 + 0x30) = *(undefined2 *)(*piVar3 + 0x28);
        fw_memcpy((void *)(*piVar3 + 0xd0),(void *)(*(int *)(*piVar3 + 0x1c) + 10),8);
        iVar9 = 0;
        do {
          iVar6 = iVar9 + 10;
          iVar9 = iVar9 + 1;
          *(undefined1 *)(*(int *)(*piVar3 + 0x1c) + iVar6) = 0;
        } while (iVar9 < 8);
        rx_copy_ccmp_hdr_fields(iVar8,*param_1);
        *(undefined1 *)(iVar4 + 0x30) = 10;
        *(int *)(iVar4 + 0x48) = local_2c;
        *(int *)(iVar4 + 0x44) = *(int *)(*piVar3 + 0x1c) + 10;
        *(int *)(iVar4 + 0x4c) = local_30;
        hif_submit_or_queue(iVar8);
        return 1;
      }
      *(int *)(iVar9 + 0x98) = *(int *)(iVar9 + 0x98) + 1;
    }
  }
  txbuf_freelist_push(piVar3);
  return 0;
}



/* ======================================================================
 * 0000c35c  rx_resolve_vif_and_link
 * ====================================================================== */

void rx_resolve_vif_and_link(int *param_1)

{
  ushort uVar1;
  bool bVar2;
  short *psVar3;
  uint uVar4;
  int iVar5;
  char *pcVar6;
  short *psVar7;
  int iVar8;
  uint uVar9;
  
  uVar1 = *(ushort *)((int)param_1 + 0x12);
  if (uVar1 == 0x74) {
    psVar3 = (short *)(*param_1 + 0x10);
  }
  else if ((uVar1 == 0xc4) || (uVar1 == 0xd4)) {
    psVar3 = (short *)0x0;
  }
  else {
    psVar3 = (short *)(*param_1 + 10);
  }
  *(undefined1 *)(param_1 + 6) = 0xf;
  *(undefined1 *)((int)param_1 + 0x17) = 0;
  if (((uVar1 & 0xf) == 0) && ((*(byte *)(*param_1 + 0x10) & 1) != 0)) {
    uVar4 = 0;
    do {
      iVar5 = uVar4 * 0x3b0 + DAT_0000c77c;
      if ((1 < *(byte *)(iVar5 + 0x19)) && (*(int *)(iVar5 + 0x1c) << 0x19 < 0)) goto LAB_0000c41e;
      uVar4 = uVar4 + 1 & 0xff;
    } while (uVar4 < 3);
    uVar4 = 0;
    do {
      iVar5 = uVar4 * 0x3b0 + DAT_0000c77c;
      if ((1 < *(byte *)(iVar5 + 0x19)) && (*(int *)(iVar5 + 0x1c) << 0x1d < 0)) goto LAB_0000c41e;
      uVar4 = uVar4 + 1 & 0xff;
    } while (uVar4 < 3);
  }
  uVar4 = 0;
  do {
    iVar5 = uVar4 * 0x3b0 + DAT_0000c77c;
    pcVar6 = (char *)(iVar5 + 0x18);
    if (1 < *(byte *)(iVar5 + 0x19)) {
      bVar2 = false;
      if ((((psVar3 != (short *)0x0) && (*psVar3 == *(short *)(iVar5 + 0x3c))) &&
          (psVar3[1] == *(short *)(iVar5 + 0x3e))) && (psVar3[2] == *(short *)(iVar5 + 0x40))) {
        bVar2 = true;
      }
      iVar8 = *param_1;
      if ((*(byte *)(iVar8 + 4) & 1) == 0) {
        if (((*(short *)(iVar8 + 4) == *(short *)(iVar5 + 0x34)) &&
            (*(short *)(iVar8 + 6) == *(short *)(iVar5 + 0x36))) &&
           (*(short *)(iVar8 + 8) == *(short *)(iVar5 + 0x38))) {
          *(char *)((int)param_1 + 0x17) = (char)uVar4;
          if (((int)((uint)*(byte *)(iVar5 + 0x1b) << 0x1d) < 0) || (*pcVar6 == '\x01')) {
            if (bVar2) {
              return;
            }
          }
          else if (psVar3 != (short *)0x0) {
            for (uVar9 = 0; uVar9 < *(ushort *)(DAT_0000c780 + 0x14); uVar9 = uVar9 + 1) {
              iVar5 = uVar9 * 0xc + DAT_0000c77c;
              iVar8 = iVar5 + DAT_0000c784;
              if ((((*(short *)(iVar8 + 0x1e) == *psVar3) &&
                   (psVar7 = (short *)(iVar5 + DAT_0000c784 + 0x20), *psVar7 == psVar3[1])) &&
                  (psVar7[1] == psVar3[2])) && (*(byte *)(iVar8 + 0x19) == uVar4)) {
                *(undefined1 *)(param_1 + 6) =
                     *(undefined1 *)(uVar9 * 0xc + DAT_0000c77c + DAT_0000c784 + 0x18);
                return;
              }
            }
          }
        }
        else if (((*pcVar6 == '\x06') || (*pcVar6 == '\x05')) &&
                ((iVar5 = uVar4 * 6 + DAT_0000c788,
                 *(short *)(iVar8 + 4) == *(short *)(iVar5 + 0x460) &&
                 ((*(short *)(iVar8 + 6) == *(short *)(iVar5 + 0x462) &&
                  (*(short *)(iVar8 + 8) == *(short *)(iVar5 + 0x464))))))) {
          *(char *)((int)param_1 + 0x17) = (char)uVar4;
        }
      }
      else {
        *(undefined1 *)(param_1 + 6) = 0;
        if ((bVar2) || (*pcVar6 == '\0')) {
LAB_0000c41e:
          *(char *)((int)param_1 + 0x17) = (char)uVar4;
          return;
        }
      }
    }
    uVar4 = uVar4 + 1 & 0xff;
    if (2 < uVar4) {
      if (*(char *)(DAT_0000c78c + 0x19) != '\x02') {
        return;
      }
      *(undefined1 *)((int)param_1 + 0x17) = 2;
      return;
    }
  } while( true );
}



/* ======================================================================
 * 0000c4f0  rx_mgmt_frame_handler
 * ====================================================================== */

void rx_mgmt_frame_handler(uint *param_1)

{
  ushort uVar1;
  short sVar2;
  byte bVar3;
  uint uVar4;
  byte *pbVar5;
  undefined1 *puVar6;
  undefined4 uVar7;
  int iVar8;
  uint uVar9;
  int iVar10;
  uint uVar11;
  short *psVar12;
  int iVar13;
  char *pcVar14;
  char *pcVar15;
  uint uVar16;
  ushort *puVar17;
  bool bVar18;
  bool bVar19;
  bool bVar20;
  bool bVar21;
  undefined8 uVar22;
  undefined4 local_58;
  int local_54;
  ushort local_4e;
  uint local_48;
  int local_44;
  int local_40;
  short *local_3c;
  uint local_38;
  int local_34;
  uint local_30;
  int local_2c;
  char *local_28;
  uint local_24;
  int local_20;
  uint local_1c;
  int local_18;
  
  puVar17 = (ushort *)0x0;
  local_40 = 0;
  local_44 = 0;
  local_48 = 0;
  local_3c = (short *)0x0;
  rx_resolve_vif_and_link(param_1);
  if ((int)(param_1[8] << 0x1c) < 0) {
    if ((int)(param_1[8] << 0x1a) < 0) {
      *(int *)(DAT_0000c790 + 0x10) = *(int *)(DAT_0000c790 + 0x10) + 1;
    }
    iVar13 = DAT_0000c790;
    *(int *)(DAT_0000c790 + 0x14) = *(int *)(DAT_0000c790 + 0x14) + 1;
    uVar22 = s64_add_s32(*(undefined4 *)(iVar13 + 0x18),*(undefined4 *)(iVar13 + 0x1c),
                         (ushort)param_1[1] + 4);
    *(undefined8 *)(iVar13 + 0x18) = uVar22;
    if ((int)(param_1[8] << 0x1a) < 0) {
      *(int *)(DAT_0000c794 + 0x18) = *(int *)(DAT_0000c794 + 0x18) + 1;
    }
  }
  uVar4 = (uint)*(byte *)((int)param_1 + 0x17);
  iVar13 = uVar4 * 0x3b0 + DAT_0000c77c;
  pcVar14 = (char *)(iVar13 + 0x18);
  uVar16 = *(uint *)(iVar13 + 0x1c);
  if (1 < uVar4) {
    uVar4 = 0;
  }
  if (*(short *)((int)param_1 + 0x12) == 0x80) {
    *DAT_0000c798 = *DAT_0000c798 + 1;
  }
  iVar8 = DAT_0000c780;
  local_30 = (uint)(ushort)param_1[1];
  local_18 = iVar13 + 0x398;
  local_1c = (uint)*(byte *)(iVar13 + 0x3a1);
  local_20 = DAT_0000c780;
  if (local_1c != 0) {
    local_24 = (uint)*(byte *)(iVar13 + 0x3a0);
    if ((local_24 != 0) && ((*pcVar14 == '\x04' || (*pcVar14 == '\x06')))) {
      for (; local_48 < *(ushort *)(DAT_0000c780 + 0x14); local_48 = local_48 + 1 & 0xff) {
        if ((char)param_1[6] == *(char *)(local_48 * 0xc + DAT_0000c77c + DAT_0000c784 + 0x18)) {
          iVar10 = local_48 * 0xc + DAT_0000c77c + DAT_0000c784;
          *(byte *)(iVar10 + 0x1b) = *(byte *)(iVar13 + 0x3a1) + *(byte *)(iVar13 + 0x3a0);
          bVar3 = *(byte *)(iVar10 + 0x1c) >> 1;
          *(byte *)(iVar10 + 0x1c) = bVar3 << 1;
          if ((bVar3 == 0) && (*(char *)(iVar10 + 0x1d) == '\0')) {
            *(ushort *)(iVar8 + 0x16) =
                 *(ushort *)(iVar8 + 0x16) & ~(ushort)(1 << *(sbyte *)(iVar10 + 0x18));
          }
          break;
        }
      }
    }
  }
  uVar9 = (uint)(ushort)param_1[4];
  if ((uVar9 & 0x3ff) >> 8 == 3) {
    local_34 = 0x1e;
    local_38 = (uint)*(ushort *)(*param_1 + 0x16);
    if ((uVar9 & 0x8f) == 0x88) {
      uVar1 = *(ushort *)(*param_1 + 0x1e);
LAB_0000c62c:
      local_3c = (short *)(uint)uVar1;
      iVar13 = local_34 + 2;
      if ((int)(uVar9 << 0x10) < 0) {
        iVar13 = local_34 + 6;
      }
      local_34 = iVar13;
      iVar13 = ((uint)local_3c & 7) * 4 + DAT_0000c790;
      *(int *)(iVar13 + 0x4c) = *(int *)(iVar13 + 0x4c) + 1;
      if ((int)((uint)(ushort)param_1[4] << 0x14) < 0) {
        *(int *)(iVar13 + 0x6c) = *(int *)(iVar13 + 0x6c) + 1;
      }
    }
  }
  else {
    local_34 = 0x18;
    local_38 = (uint)*(ushort *)(*param_1 + 0x16);
    if ((uVar9 & 0x8f) == 0x88) {
      uVar1 = *(ushort *)(*param_1 + 0x18);
      goto LAB_0000c62c;
    }
  }
  uVar9 = (uint)(ushort)param_1[4];
  if ((int)(uVar9 << 0x12) < 0) {
    param_1[8] = param_1[8] | 0x1000;
  }
  uVar11 = uVar9 & 0xf;
  if (uVar11 == 4) goto LAB_0000c6fa;
  if ((uVar9 & 0x3ff) >> 8 == 3) {
LAB_0000c68a:
    puVar17 = (ushort *)(*param_1 + 4);
  }
  else if ((int)(uVar9 << 0x16) < 0) {
    puVar17 = (ushort *)(*param_1 + 10);
  }
  else {
    if ((int)(uVar9 << 0x17) < 0) goto LAB_0000c68a;
    puVar17 = (ushort *)(*param_1 + 0x10);
  }
  uVar9 = (uint)*puVar17;
  if (uVar9 != 0) goto LAB_0000c6a6;
  if (puVar17[1] != 0) goto LAB_0000c6a6;
  bVar19 = puVar17[2] == 0;
LAB_0000c6a4:
  if (!bVar19) {
LAB_0000c6a6:
    iVar13 = uVar4 * 0x98 + DAT_0000c788;
    if ((((uVar9 == *(ushort *)(iVar13 + 0x482)) && (puVar17[1] == *(ushort *)(iVar13 + 0x484))) &&
        (puVar17[2] == *(ushort *)(iVar13 + 0x486))) ||
       (((iVar13 = (uint)*(byte *)((int)param_1 + 0x17) * 6 + DAT_0000c788,
         uVar9 == *(ushort *)(iVar13 + 0x460) && (puVar17[1] == *(ushort *)(iVar13 + 0x462))) &&
        (puVar17[2] == *(ushort *)(iVar13 + 0x464))))) {
      param_1[8] = param_1[8] | 0x800;
    }
LAB_0000c6fa:
    if (*pcVar14 == '\a') goto LAB_0000cf74;
    bVar3 = 0;
    if (((int)(param_1[8] << 0x14) < 0) && (uVar11 == 8)) {
      bVar3 = 1;
    }
    local_28 = pcVar14 + 0x3a0;
    pcVar14[0x3ac] = bVar3 | pcVar14[0x3ac];
    bVar19 = *DAT_0000c79c << 0xd < 0;
LAB_0000c724:
    if (!bVar19) {
      uVar4 = *param_1;
      if (((*(ushort *)(uVar4 + 4) == DAT_0000c7a0) && (*(ushort *)(uVar4 + 6) == DAT_0000c7a0)) &&
         (*(ushort *)(uVar4 + 8) == DAT_0000c7a0)) {
        param_1[8] = param_1[8] | 0x40000;
      }
      if (-1 < *(int *)(pcVar14 + 4) << 0x18) goto LAB_0000c7b2;
      uVar9 = (uint)*(ushort *)((int)param_1 + 0x12);
      bVar20 = uVar9 == 0x80;
LAB_0000c750:
      bVar19 = true;
      if (bVar20) goto LAB_0000c6a4;
      if (uVar9 == 0x40) {
        pbVar5 = ie_find((byte *)(uVar4 + 0x18),local_30,0,0);
        if (pbVar5 != (byte *)0x0) {
          uVar22 = fw_mem_equal(pbVar5 + 2,pbVar5[1],s_DIRECT__0000c7a4,7);
          uVar9 = (uint)((ulonglong)uVar22 >> 0x20);
          bVar19 = true;
          uVar4 = 0;
          if ((int)uVar22 == 0) goto LAB_0000c6a4;
        }
      }
      else {
        bVar19 = true;
        if ((int)(param_1[8] << 0xd) < 0) goto LAB_0000c724;
      }
LAB_0000c7b2:
      pcVar15 = DAT_0000cbac;
      if (-1 < *DAT_0000cba4 << 0x1a) {
        pcVar15 = pcVar14;
        if (-1 < (int)(uVar16 << 0x1d)) goto LAB_0000c962;
        sVar2 = *(short *)((int)param_1 + 0x12);
        if (sVar2 == 0x40) goto LAB_0000c962;
        if ((((int)((uint)(ushort)param_1[4] << 0x13) < 0) && (sVar2 != 0)) && (sVar2 != 0xb0)) {
          uVar4 = (uint)(byte)param_1[6];
          if ((uVar4 != 0) && (uVar4 < 0xf)) {
            if ((((int)((uint)*(ushort *)
                               ((uint)*(byte *)((int)param_1 + 0x17) * 0x104 + DAT_0000cbb0 + 0x5a)
                       << 0x19) < 0) && (((uint)*(ushort *)(pcVar14 + 0x144) & 1 << uVar4) != 0)) &&
               ((sVar2 == 0x88 || (sVar2 == 200)))) {
              *(ushort *)(pcVar14 + 0x148) = *(ushort *)(pcVar14 + 0x148) | (ushort)(1 << uVar4);
            }
            *(ushort *)(pcVar14 + 0x144) =
                 (ushort)(1 << (sbyte)param_1[6]) | *(ushort *)(pcVar14 + 0x144);
          }
          if ((-1 < *(int *)(local_18 + 0xc) << 0x1b) || (*(short *)((int)param_1 + 0x12) != 0x48))
          goto LAB_0000c930;
        }
        else {
          *(ushort *)(pcVar14 + 0x144) =
               *(ushort *)(pcVar14 + 0x144) & ~(ushort)(1 << (sbyte)param_1[6]);
          *(ushort *)(pcVar14 + 0x148) =
               *(ushort *)(pcVar14 + 0x148) & ~(ushort)(1 << (sbyte)param_1[6]);
          iVar13 = local_48 * 0xc + DAT_0000cbb4 + DAT_0000cbb8;
          bVar3 = *(byte *)(iVar13 + 0x1c);
          *(byte *)(iVar13 + 0x1c) = bVar3 & 0xfd;
          if (((bVar3 & 0xfd) == 0) && (*(char *)(iVar13 + 0x1d) == '\0')) {
            *(ushort *)(local_20 + 0x16) =
                 *(ushort *)(local_20 + 0x16) & ~(ushort)(1 << (sbyte)param_1[6]);
          }
          if (((-1 < *(int *)(local_18 + 0xc) << 0x1b) || (*(short *)((int)param_1 + 0x12) != 0x48))
             || (local_28[0xd] != '\0')) goto LAB_0000c930;
        }
        local_44 = 1;
LAB_0000c930:
        if (*(short *)(pcVar14 + 0x16) != 0) {
          *(ushort *)(pcVar14 + 0x16) =
               (*(ushort *)(pcVar14 + 0x146) |
               ~*(ushort *)(pcVar14 + 0x144) | *(ushort *)(pcVar14 + 0x148)) &
               *(ushort *)(pcVar14 + 0x14);
          evt_flags_set(DAT_0000cbbc,0x200000);
        }
        bVar19 = local_44 == 1;
        goto LAB_0000c960;
      }
      sVar2 = *(short *)((int)param_1 + 0x12);
      if ((sVar2 == 0x50) || (sVar2 == 0x80)) {
        if ((byte)param_1[5] < *(byte *)(DAT_0000cba8 + 0x19)) goto LAB_0000cfcc;
        if (sVar2 == 0x40) goto LAB_0000c7d8;
        if (sVar2 == 0x80) {
          *(int *)(DAT_0000cbac + 4) = *(int *)(DAT_0000cbac + 4) + 1;
          param_1[8] = param_1[8] | 0x80;
          iVar13 = measure_is_state5();
          if (iVar13 != 0) {
            param_1[8] = param_1[8] | 0x80000000;
          }
          pcVar14 = pcVar15;
          if ((*(byte *)((int)param_1 + 0x17) < 2) && ((int)(param_1[8] << 0x14) < 0)) {
            *(int *)(pcVar15 + 0x14) = *(int *)(pcVar15 + 0x14) + 1;
            vif_bss_event_timer_update(*(undefined1 *)((int)param_1 + 0x17));
          }
        }
        else if (sVar2 == 0x50) {
          *(int *)(DAT_0000cbac + 8) = *(int *)(DAT_0000cbac + 8) + 1;
        }
      }
      else {
        if (sVar2 != 0x40) goto LAB_0000cfcc;
LAB_0000c7d8:
        if ((int)(uVar16 << 0x1d) < 0) goto LAB_0000cafe;
      }
      uVar22 = rx_beacon_validate_channel(param_1);
      uVar9 = (uint)((ulonglong)uVar22 >> 0x20);
      bVar20 = (int)uVar22 == 0;
      uVar4 = 0;
      if (!bVar20) {
        if (-1 < (int)(param_1[8] << 0x15)) goto LAB_0000cf74;
        uVar22 = syn_scan_filter_rx_frame(param_1);
        uVar9 = (uint)((ulonglong)uVar22 >> 0x20);
        bVar20 = (int)uVar22 == 0;
        uVar4 = 0;
        if (!bVar20) goto LAB_0000cf74;
      }
      goto LAB_0000c750;
    }
  }
LAB_0000cfcc:
  *(int *)(DAT_0000d000 + 4) = *(int *)(DAT_0000d000 + 4) + 1;
  rx_buf_free(*param_1);
  return;
LAB_0000cafe:
  iVar13 = ap_send_probe_response(param_1);
  bVar20 = true;
  if (iVar13 == 0) {
LAB_0000ca3a:
    bVar19 = true;
    if (bVar20) {
LAB_0000c960:
      pcVar15 = pcVar14;
      if (!bVar19) {
LAB_0000c962:
        uVar4 = *param_1;
        local_2c = DAT_0000cbc0;
        pcVar14 = pcVar15;
        if ((*(byte *)(uVar4 + 4) & 1) != 0) goto code_r0x0000c96e;
        if (((*(short *)(uVar4 + 4) == *(short *)(pcVar15 + 0x1c)) &&
            (*(short *)(uVar4 + 6) == *(short *)(pcVar15 + 0x1e))) &&
           (*(short *)(uVar4 + 8) == *(short *)(pcVar15 + 0x20))) goto LAB_0000caa6;
        psVar12 = (short *)((uint)*(byte *)((int)param_1 + 0x17) * 6 + DAT_0000cbcc + 0x460);
        bVar20 = *(short *)(uVar4 + 4) == *psVar12;
LAB_0000ca94:
        bVar19 = false;
        if (((bVar20) && (bVar19 = false, *(short *)(uVar4 + 6) == psVar12[1])) &&
           (bVar19 = false, *(short *)(uVar4 + 8) == psVar12[2])) {
LAB_0000caa6:
          uVar4 = param_1[8];
          param_1[8] = uVar4 | 0x10000;
          if (-1 < (int)(uVar16 << 0x19)) {
            psVar12 = (short *)(uint)(ushort)param_1[4];
            if (((uint)psVar12 & 0xf) == 4) goto LAB_0000caf2;
            if (pcVar14[0x1cf] != '\0') goto LAB_0000caf2;
            if ((int)(uVar4 << 0x14) < 0) goto LAB_0000caf2;
            if ((((int)(uVar16 << 0x1d) < 0) && (*(short *)((int)param_1 + 0x12) == 0x40)) &&
               ((*puVar17 == DAT_0000cbd0 &&
                ((puVar17[1] == DAT_0000cbd0 && (puVar17[2] == DAT_0000cbd0)))))) goto LAB_0000caf8;
            bVar19 = local_28[0xe] == '\0';
            do {
              bVar20 = true;
              if (bVar19) goto LAB_0000ca3a;
LAB_0000caf2:
              if (*(short *)((int)param_1 + 0x12) == 0x40) {
LAB_0000caf8:
                if ((uVar16 & 0xf) >> 2 != 0) goto LAB_0000cafe;
              }
              if (((uint)psVar12 & 0x8f) != 0x88) {
                uVar4 = 0x11;
LAB_0000cbfc:
                pcVar15 = pcVar14;
                if ((((ushort)param_1[4] & 0xf) == 4) || (*(short *)((int)param_1 + 0x12) == 200))
                goto LAB_0000cc52;
                iVar13 = rx_dup_cache_check((ushort)param_1[4],*param_1 + 10,local_38,
                                            (uint)*(byte *)((int)param_1 + 0x17) << 8 | uVar4);
                if (iVar13 != 0) {
                  *(int *)(pcVar14 + 0x70) = *(int *)(pcVar14 + 0x70) + 1;
                  if (uVar4 != 0x11) {
                    iVar13 = (uVar4 & 7) * 4 + DAT_0000cfd8;
                    *(int *)(iVar13 + 0xec) = *(int *)(iVar13 + 0xec) + 1;
                  }
                  goto LAB_0000cfcc;
                }
                if (((uVar16 & 1) != 0) && (-1 < (int)((uint)(ushort)param_1[4] << 0x15))) {
                  ps_on_tx_complete(param_1,local_3c);
                }
                goto LAB_0000cc52;
              }
              if ((uVar16 & 0x1d) == 0) {
LAB_0000cbe8:
                uVar4 = (uint)local_3c & 0xf;
                if ((uVar4 < 8) || (uVar4 == 0x11)) goto LAB_0000cbfc;
                goto LAB_0000cfcc;
              }
              uVar4 = bab_find_session(*(undefined1 *)((int)param_1 + 0x17));
              if (uVar4 < 4) {
                iVar13 = uVar4 * 0x28 + DAT_0000cbd4;
                uVar4 = (uint)*(ushort *)(iVar13 + 0x3b2);
                if (uVar4 != 0) {
                  timer_start(iVar13 + 0x3b4,uVar4 << 10);
                }
                goto LAB_0000cbe8;
              }
              if (((-1 < (int)(param_1[8] << 0x1c)) || (((uint)local_3c & 0x7f) >> 5 != 0)) &&
                 (((uint)local_3c & 0x7f) >> 5 != 3)) goto LAB_0000cbe8;
              psVar12 = local_3c;
              uVar22 = bab_session_try_alloc();
              uVar4 = (uint)((ulonglong)uVar22 >> 0x20);
              bVar20 = (int)uVar22 == 0;
              if (!bVar20) goto LAB_0000ca94;
              puVar6 = (undefined1 *)lmc_msg_alloc();
              bVar19 = puVar6 == (undefined1 *)0x0;
              pcVar14 = (char *)0x0;
              if (!bVar19) {
                *puVar6 = 6;
                puVar6[0x29] = 8;
                fw_memcpy(puVar6 + 8,(void *)(*param_1 + 10),6);
                puVar6[4] = (byte)local_3c & 0xf;
                *(undefined2 *)(puVar6 + 2) = 0x26;
                bVar3 = *(byte *)((int)param_1 + 0x16);
                puVar6[1] = bVar3;
                if (*(char *)((int)param_1 + 7) == '\x04') {
                  puVar6[1] = bVar3 | 0x80;
                }
                puVar6[0x28] = *(undefined1 *)((int)param_1 + 0x17);
                evt_flags_set(DAT_0000cfd4,0x400000);
                goto LAB_0000cfcc;
              }
            } while( true );
          }
          goto LAB_0000cf74;
        }
        goto LAB_0000c9ea;
      }
      goto LAB_0000cfcc;
    }
    goto LAB_0000cafe;
  }
  goto LAB_0000cf74;
code_r0x0000c96e:
  *(int *)(pcVar15 + 0x78) = *(int *)(pcVar15 + 0x78) + 1;
  uVar4 = (uint)(ushort)param_1[4];
  bVar19 = true;
  if ((uVar4 & 0xf) == 4) goto LAB_0000c960;
  if ((int)(uVar16 << 0x19) < 0) goto LAB_0000cf74;
  if (((((int)(uVar4 << 0x16) < 0) &&
       (uVar9 = *param_1, *(short *)(uVar9 + 0x10) == *(short *)(pcVar15 + 0x1c))) &&
      (*(short *)(uVar9 + 0x12) == *(short *)(pcVar15 + 0x1e))) &&
     (bVar19 = true, *(short *)(uVar9 + 0x14) == *(short *)(pcVar15 + 0x20))) goto LAB_0000c960;
  if (-1 < (int)(param_1[8] << 0xd)) {
    param_1[8] = param_1[8] | 0x20000;
  }
  if ((int)(param_1[8] << 0x14) < 0) {
    uVar9 = (uint)*(ushort *)((int)param_1 + 0x12);
    if ((uVar9 == 0x40) && ((int)(uVar16 << 0x1c) < 0)) goto LAB_0000cafe;
    bVar20 = SBORROW4(uVar9,0xc4);
    bVar18 = (int)(uVar9 - 0xc4) < 0;
    bVar19 = uVar9 == 0xc4;
    goto LAB_0000c9c0;
  }
  if (*(short *)((int)param_1 + 0x12) == 0x80) {
    if ((int)(uVar16 << 0x1d) < 0) {
      bVar19 = *(int *)(pcVar15 + 4) << 0xf < 0;
      goto LAB_0000c9e0;
    }
    pcVar14 = pcVar15 + 0x1c0;
    if (pcVar15[0x1cf] != '\0') goto LAB_0000ca0a;
    if (-1 < (int)(uVar16 << 0x1c)) goto LAB_0000cfcc;
    iVar13 = rx_probe_resp_matches_our_ssid(param_1);
    if (iVar13 == 1) goto LAB_0000cf74;
  }
  bVar19 = *(short *)((int)param_1 + 0x12) == 0x40;
  if (!bVar19) {
LAB_0000c9ea:
    while( true ) {
      if (!bVar19) goto LAB_0000cfcc;
      bVar19 = true;
      if (*pcVar14 == '\x06') break;
      if ((*pcVar14 != '\x04') ||
         (bVar19 = (int)((uint)*(byte *)(DAT_0000cbc4 + 5) << 0x1e) < 0, !bVar19))
      goto LAB_0000cf74;
LAB_0000c9e0:
      if (bVar19) goto LAB_0000cfcc;
      iVar13 = rx_beacon_update_erp_ht_flags(param_1);
      bVar19 = iVar13 == 0;
    }
    goto LAB_0000c960;
  }
  bVar20 = (uVar16 & 0xf) >> 2 == 0;
  goto LAB_0000ca3a;
  while( true ) {
    bVar18 = iVar13 < 0;
    bVar19 = true;
    if (!bVar21) break;
LAB_0000c9c0:
    if (bVar19) goto LAB_0000cc52;
    if (bVar18 == bVar20) {
      bVar20 = SBORROW4(uVar9,0xd4);
      bVar18 = (int)(uVar9 - 0xd4) < 0;
      bVar19 = true;
      if (uVar9 == 0xd4) goto LAB_0000c9c0;
      bVar20 = SBORROW4(uVar9,0xe4);
      bVar18 = (int)(uVar9 - 0xe4) < 0;
      bVar19 = true;
      if (uVar9 == 0xe4) goto LAB_0000c9c0;
      bVar20 = SBORROW4(uVar9,0xf4);
      iVar13 = uVar9 - 0xf4;
      bVar21 = uVar9 == 0xf4;
    }
    else {
      bVar20 = SBORROW4(uVar9,0x80);
      bVar18 = (int)(uVar9 - 0x80) < 0;
      bVar19 = true;
      if (uVar9 == 0x80) goto LAB_0000c9c0;
      bVar20 = SBORROW4(uVar9,0xa4);
      bVar18 = (int)(uVar9 - 0xa4) < 0;
      bVar19 = true;
      if (uVar9 == 0xa4) goto LAB_0000c9c0;
      bVar20 = SBORROW4(uVar9,0xb4);
      iVar13 = uVar9 - 0xb4;
      bVar21 = uVar9 == 0xb4;
    }
  }
  if ((uVar16 & 0x101) == 0x101) {
    ps_clear_ind_counters(*(undefined1 *)((int)param_1 + 0x17),uVar4 & 0x2000);
  }
LAB_0000cc52:
  if ((((uVar16 & 0x104) == 0) && (*(short *)((int)param_1 + 0x12) != 0x50)) &&
     (*(short *)((int)param_1 + 0x12) != 0x80)) goto LAB_0000cfcc;
  if (((int)(uVar16 << 0x1c) < 0) &&
     (bab_set_session_state_by_mac(*param_1 + 10,(ushort)param_1[4] & 0x1000),
     *(short *)((int)param_1 + 0x12) == 0x90)) {
    hif_mark_confirm_pending(*param_1 + 10);
    goto LAB_0000cfcc;
  }
  uVar4 = (uint)*(ushort *)((int)param_1 + 0x12);
  if ((*(ushort *)((int)param_1 + 0x12) & 0xf) == 0) {
    if (*DAT_0000cfe0 << 0x1a < 0) goto LAB_0000cf74;
    if ((int)((uint)(ushort)param_1[4] << 0x11) < 0) {
      if ((uVar4 != 0xb0) || (*(int *)(pcVar15 + 0x120) << 0x1d < 0)) {
        iVar13 = frame_is_unprotected_mgmt(*param_1);
        if ((iVar13 == 0) || ((*(uint *)(pcVar15 + 0x120) & 1) == 0)) goto LAB_0000cfcc;
        local_40 = key_lookup_for_frame(*(undefined1 *)((int)param_1 + 0x17),*param_1 + 10,0xf);
        goto LAB_0000cf68;
      }
      local_40 = key_lookup_for_frame(*(undefined1 *)((int)param_1 + 0x17),*param_1 + 10,0xf);
      param_1[8] = param_1[8] | (uint)(byte)pcVar15[0xfa] << 0x14;
      if ((local_40 == 0) &&
         (pbVar5 = (byte *)(*param_1 + local_34),
         local_40 = key_lookup_for_frame
                              (*(undefined1 *)((int)param_1 + 0x17),*param_1 + 10,
                               (*pbVar5 & 1) << 2 | pbVar5[3] >> 6 | 0x10), local_40 != 0)) {
        if (((*(byte *)(*param_1 + 4) & 1) != 0) || (*(char *)(local_40 + 1) == '\0')) {
          uVar4 = param_1[8];
          goto LAB_0000cf6c;
        }
        local_40 = 0;
      }
      goto LAB_0000cf74;
    }
    if ((uVar4 != 0x50) && (uVar4 != 0x80)) goto LAB_0000cfb4;
    if ((*DAT_0000cfe0 << 0x17 < 0) &&
       (*(char *)((int)param_1 + 0x17) == *(char *)(local_2c + 0x17))) {
      iVar13 = rx_beacon_join_match(param_1,local_30);
      if (((iVar13 != 0) && (*(short *)((int)param_1 + 0x12) == 0x80)) &&
         ((int)(param_1[8] << 0x14) < 0)) {
        ps_beacon_rx_update_timing(param_1);
      }
      goto LAB_0000cfcc;
    }
    if ((int)(uVar16 << 0x1c) < 0) {
      iVar13 = rx_probe_resp_matches_our_ssid(param_1);
      if (iVar13 != 1) goto LAB_0000cfcc;
      goto LAB_0000cf74;
    }
    if (((int)(uVar16 << 0x17) < 0) && (uVar4 == 0x80)) {
      if (-1 < (int)(param_1[8] << 0x14)) goto LAB_0000cfcc;
      *(int *)(DAT_0000cfe4 + 0x14) = *(int *)(DAT_0000cfe4 + 0x14) + 1;
      vif_bss_event_timer_update(*(undefined1 *)((int)param_1 + 0x17));
      ps_beacon_rx_update_timing(param_1);
      uVar7 = ie_find_in_frame(*param_1,local_30,5,0);
      uVar4 = rx_beacon_check_tim_for_us(*(undefined1 *)((int)param_1 + 0x17),uVar7);
      uVar16 = 0;
      do {
        iVar13 = ie_find_in_frame(*param_1,local_30,0x28,uVar16);
        if ((iVar13 == 0) || (*(char *)(iVar13 + 1) != '\x06')) {
          *(undefined1 *)(DAT_0000cfe8 + uVar16 + DAT_0000cfec + 0x1e) = 0;
        }
        else {
          measure_store_slot_params(iVar13,uVar16);
        }
        uVar16 = uVar16 + 1 & 0xff;
      } while (uVar16 < 2);
      iVar13 = ie_find_in_frame(*param_1,local_30,0x2a,0);
      if ((((*(byte *)((int)param_1 + 0x17) < 2) &&
           (*(int *)((uint)*(byte *)((int)param_1 + 0x17) * 0xc + DAT_0000cff0 + 0x358) << 0x1e < 0)
           ) && (iVar13 != 0)) && (*(char *)(iVar13 + 1) == '\x01')) {
        if ((int)((uint)*(byte *)(iVar13 + 2) << 0x1e) < 0) {
          pcVar15[0xe] = '\x01';
        }
        else {
          pcVar15[0xe] = '\0';
        }
        pas_pick_lowest_rate_from_mask(pcVar15);
      }
      iVar13 = DAT_0000cff4;
      if (*(char *)(DAT_0000cff4 + 8) == '\x01') {
        timer_cancel(DAT_0000cff8);
        *(undefined1 *)(iVar13 + 8) = 0;
      }
      iVar8 = beacon_pick_soonest_vif();
      if (0 < iVar8) {
        timer_start(DAT_0000cff8,iVar8);
        *(undefined1 *)(iVar13 + 8) = 1;
      }
      if ((uVar4 & 0xff) != 0) {
        param_1[8] = param_1[8] | 0x100;
      }
      if (uVar4 >> 8 != 0) {
        param_1[8] = param_1[8] | 0x200;
      }
      ps_on_beacon_rx(*(undefined1 *)((int)param_1 + 0x17),uVar4);
LAB_0000ca0a:
      if (((short)param_1[7] == *(short *)(local_2c + 0x14)) &&
         (iVar13 = rx_beacon_validate_channel(param_1), iVar13 != 0)) {
        uVar4 = param_1[8];
        uVar16 = 0x80;
      }
      else {
        uVar4 = param_1[8];
        uVar16 = DAT_0000cbc8;
      }
      goto LAB_0000cf70;
    }
    if (uVar4 != 0x50) goto LAB_0000cfcc;
LAB_0000cf7e:
    iVar13 = DAT_0000cffc;
    uVar4 = *param_1;
    if ((*(short *)(DAT_0000cffc + 0xc) == *(short *)(uVar4 + 10)) &&
       (((*(short *)(DAT_0000cffc + 0xe) == *(short *)(uVar4 + 0xc) &&
         (*(short *)(DAT_0000cffc + 0x10) == *(short *)(uVar4 + 0xe))) &&
        (*(ushort *)(DAT_0000cffc + 0x12) == local_38)))) goto LAB_0000cfcc;
    *(short *)(DAT_0000cffc + 0xc) = *(short *)(uVar4 + 10);
    *(undefined2 *)(iVar13 + 0xe) = *(undefined2 *)(*param_1 + 0xc);
    *(undefined2 *)(iVar13 + 0x10) = *(undefined2 *)(*param_1 + 0xe);
    *(short *)(iVar13 + 0x12) = (short)local_38;
  }
  else if ((uVar4 & 0xf) == 4) {
    if (uVar4 == 0x84) {
      if (1 < *(byte *)((int)param_1 + 0x17)) goto LAB_0000cfcc;
      if ((*(byte *)(DAT_0000cfdc + 0x10) & 1) != 0) {
        uVar4 = *param_1;
        local_58._0_3_ =
             CONCAT12((byte)((ushort)*(undefined2 *)(uVar4 + 0x10) >> 0xc),(undefined2)local_58);
        fw_memcpy(&local_54,(void *)(uVar4 + 10),6);
        local_4e = *(ushort *)(uVar4 + 0x12) >> 4;
        local_58 = (short *)CONCAT31(local_58._1_3_,3);
        bab_rx_ba_session_ctl(*(undefined1 *)((int)param_1 + 0x17),&local_58);
        goto LAB_0000cf74;
      }
    }
    else if (uVar4 != 0xa4) goto LAB_0000cfcc;
  }
  else {
    if ((uVar4 & 0xf) != 8) goto LAB_0000cfcc;
    bVar20 = SBORROW4(uVar4,0x68);
    iVar13 = uVar4 - 0x68;
    bVar19 = uVar4 == 0x68;
    do {
      if (bVar19) goto LAB_0000cef8;
      if (iVar13 < 0 == bVar20) {
        if ((uVar4 == 0x78) || (uVar4 == 200)) goto LAB_0000cef8;
        goto LAB_0000cefc;
      }
      bVar20 = SBORROW4(uVar4,0x48);
      iVar13 = uVar4 - 0x48;
      bVar19 = true;
    } while (uVar4 == 0x48);
    if (uVar4 == 0x58) {
LAB_0000cef8:
      if (-1 < (int)(uVar16 << 0x1d)) goto LAB_0000cfcc;
    }
LAB_0000cefc:
    if (-1 < (int)((uint)(ushort)param_1[4] << 0x11)) goto LAB_0000cf74;
    bVar19 = true;
    if ((*(byte *)(*param_1 + 4) & 1) == 0) {
      bVar19 = false;
      local_40 = key_lookup_for_frame(*(undefined1 *)((int)param_1 + 0x17),*param_1 + 10,0xf);
      param_1[8] = (uint)(byte)pcVar15[0xfa] << 0x14 | param_1[8];
      if (local_40 != 0) goto LAB_0000cf74;
    }
    pbVar5 = (byte *)(*param_1 + local_34);
    local_40 = key_lookup_for_frame
                         (*(undefined1 *)((int)param_1 + 0x17),*param_1 + 10,
                          (*pbVar5 & 1) << 2 | pbVar5[3] >> 6 | 0x10);
    if (((local_40 != 0) && (!bVar19)) && (*(char *)(local_40 + 1) != '\0')) {
      local_40 = 0;
    }
LAB_0000cf68:
    uVar4 = param_1[8];
LAB_0000cf6c:
    uVar16 = (uint)(byte)pcVar15[0xfa] << 0x14;
LAB_0000cf70:
    param_1[8] = uVar4 | uVar16;
LAB_0000cf74:
    if ((*(short *)((int)param_1 + 0x12) == 0x80) || (*(short *)((int)param_1 + 0x12) == 0x50))
    goto LAB_0000cf7e;
  }
LAB_0000cfb4:
  local_58 = local_3c;
  local_54 = local_40;
  iVar13 = rx_decrypt_and_verify(param_1,local_30,local_34,local_38);
  if (iVar13 != 0) {
    return;
  }
  goto LAB_0000cfcc;
}



/* ======================================================================
 * 0000d010  tx_ctx_free_inner
 * ====================================================================== */

void tx_ctx_free_inner(int param_1)

{
  int iVar1;
  int iVar2;
  
  iVar2 = DAT_0000d40c;
  iVar1 = *(int *)(DAT_0000d40c + 0x10);
  *(int *)(iVar1 * 4 + DAT_0000d40c + 0x14) = param_1;
  *(uint *)(iVar2 + 0x10) = iVar1 + 1U & 0x3f;
  *(uint *)(param_1 + 0x2c) = *(uint *)(param_1 + 0x2c) | 0x4000;
  if (*(short *)(param_1 + 0x1c) == 0x16) {
    *(ushort *)(param_1 + 0x50) = *(ushort *)(param_1 + 0x50) | 2;
  }
  else {
    evt_flags_set(DAT_0000d410,0x200000);
  }
  if (((*(char *)(param_1 + -1) != '\0') || (-1 < (int)((uint)*(ushort *)(param_1 + 10) << 0x1c)))
     || (iVar2 = hif_confirm_coalesce_hold(), iVar2 == 0)) {
    evt_flags_set(DAT_0000d410,0x100000);
  }
  return;
}



/* ======================================================================
 * 0000d074  tx_ctx_free_locked
 * ====================================================================== */

void tx_ctx_free_locked(undefined4 param_1)

{
  undefined4 uVar1;
  
  uVar1 = irq_fiq_disable_save();
  tx_ctx_free_inner(param_1);
  irq_fiq_restore(uVar1);
  return;
}



/* ======================================================================
 * 0000d08c  tx_ctx_alloc_init
 * ====================================================================== */

/* tx_ctx_alloc_init(class, queue_id, flag) -- pop the internal TX descriptor
   free list and initialise the descriptor.  Returns 0 when the pool is empty.
   
   Pool: exactly **3** descriptors of 0x170 bytes, built by tx_ctx_pool_init
   (0x00012574).  NOT the 30 host input buffers.
   
   *** POOL EXHAUSTION IS COMPLETELY UNOBSERVABLE. ***
   All ten call sites were checked.  Every one treats a 0 return as a silent
   drop: no counter is incremented, no assert fires, no indication is sent, and
   no MIB exports a failure count.  Some callers set a per-vif retry bit
   (g_lmc_ctx+0xB) so the frame is re-attempted later, which is graceful, but
   nothing anywhere records that it happened.
   
   Consequence for debugging: if the three-deep pool ever throttles TX, it will
   be **invisible to the host** by any means short of a bulk memory read
   (WSM 0x0000) of the free-list head at 0x04009080 sampled during a transfer.
   There is no counter to read via MIB 0x1037 or 0x1036.
   
   Callers, all now named: tx_send_null_data (0x3D38), tx_send_qos_null (0x3FE0),
   tx_send_ps_poll (0x4076), tx_send_template_frame (0x411A),
   ap_send_probe_response (0x5366), syn_scan_build_probe_req (0x141B0), and
   0x761C / 0x76D0 / 0x77B8 / 0x79F4 in tx_ptcs.c. */

int tx_ctx_alloc_init(int param_1,undefined1 param_2,undefined1 param_3)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  
  iVar2 = DAT_0000d414;
  iVar1 = DAT_0000d40c;
  iVar3 = *(int *)(DAT_0000d414 + 0x14);
  if (iVar3 != 0) {
    if (param_1 == 0) {
      *(char *)(DAT_0000d40c + 5) = *(char *)(DAT_0000d40c + 5) + '\x01';
    }
    else {
      *(char *)(DAT_0000d40c + 4) = *(char *)(DAT_0000d40c + 4) + '\x01';
    }
    *(undefined4 *)(iVar2 + 0x14) = *(undefined4 *)(iVar3 + 4);
    *(undefined1 *)(iVar3 + 0xd) = param_2;
    *(undefined4 *)(iVar3 + 0x4c) = 0;
    *(char *)(iVar3 + 0x53) = (char)param_1;
    *(undefined1 *)(iVar3 + 0x52) = param_3;
    *(undefined2 *)(iVar3 + 0x50) = *(undefined2 *)(iVar1 + 8);
    *(short *)(iVar1 + 8) = *(short *)(iVar1 + 8) + 1;
    *(undefined1 *)(iVar3 + 0xf) = 0;
    *(undefined1 *)(iVar3 + 0xa7) = *(undefined1 *)(iVar3 + 0x52);
    *(undefined4 *)(iVar3 + 0x58) = 0;
    *(undefined2 *)(iVar3 + 0x70) = 0xfe;
    *(undefined2 *)(iVar3 + 0x72) = 0;
    *(undefined2 *)(iVar3 + 0xa4) = 0;
    *(undefined4 *)(iVar3 + 0x80) = 1;
    uVar4 = *(undefined4 *)(DAT_0000d418 + 0x44);
    *(undefined4 *)(iVar3 + 0x90) = 0;
    *(undefined4 *)(iVar3 + 0x98) = uVar4;
    *(undefined1 *)(iVar3 + 0x60) = *(undefined1 *)(DAT_0000d41c + (uint)*(byte *)(iVar3 + 0xd));
  }
  return iVar3;
}



/* ======================================================================
 * 0000d0f8  tx_ctx_alloc_mgmt
 * ====================================================================== */

/* WARNING: Removing unreachable block (ram,0x0000d09c) */

int tx_ctx_alloc_mgmt(void)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  
  iVar2 = DAT_0000d414;
  iVar1 = DAT_0000d40c;
  iVar3 = *(int *)(DAT_0000d414 + 0x14);
  if (iVar3 != 0) {
    *(char *)(DAT_0000d40c + 5) = *(char *)(DAT_0000d40c + 5) + '\x01';
    *(undefined4 *)(iVar2 + 0x14) = *(undefined4 *)(iVar3 + 4);
    *(undefined1 *)(iVar3 + 0xd) = 0;
    *(undefined4 *)(iVar3 + 0x4c) = 0;
    *(undefined1 *)(iVar3 + 0x53) = 0;
    *(undefined1 *)(iVar3 + 0x52) = 1;
    *(undefined2 *)(iVar3 + 0x50) = *(undefined2 *)(iVar1 + 8);
    *(short *)(iVar1 + 8) = *(short *)(iVar1 + 8) + 1;
    *(undefined1 *)(iVar3 + 0xf) = 0;
    *(undefined1 *)(iVar3 + 0xa7) = *(undefined1 *)(iVar3 + 0x52);
    *(undefined4 *)(iVar3 + 0x58) = 0;
    *(undefined2 *)(iVar3 + 0x70) = 0xfe;
    *(undefined2 *)(iVar3 + 0x72) = 0;
    *(undefined2 *)(iVar3 + 0xa4) = 0;
    *(undefined4 *)(iVar3 + 0x80) = 1;
    uVar4 = *(undefined4 *)(DAT_0000d418 + 0x44);
    *(undefined4 *)(iVar3 + 0x90) = 0;
    *(undefined4 *)(iVar3 + 0x98) = uVar4;
    *(undefined1 *)(iVar3 + 0x60) = *(undefined1 *)(DAT_0000d41c + (uint)*(byte *)(iVar3 + 0xd));
  }
  return iVar3;
}



/* ======================================================================
 * 0000d100  tx_ctx_free
 * ====================================================================== */

void tx_ctx_free(int param_1)

{
  int iVar1;
  
  iVar1 = DAT_0000d414;
  *(undefined4 *)(param_1 + 4) = *(undefined4 *)(DAT_0000d414 + 0x14);
  *(undefined2 *)(param_1 + 0x70) = 0xff;
  *(uint *)(param_1 + 0x80) = *(uint *)(param_1 + 0x80) | 0x20000;
  *(int *)(iVar1 + 0x14) = param_1;
  if (*(char *)(param_1 + 0x53) == '\0') {
    *(char *)(DAT_0000d40c + 5) = *(char *)(DAT_0000d40c + 5) + -1;
  }
  else {
    *(char *)(DAT_0000d40c + 4) = *(char *)(DAT_0000d40c + 4) + -1;
  }
  if (*(char *)(DAT_0000d420 + 0xb) != '\0') {
    tx_pending_ctrl_frame_dispatch();
  }
  if ((int)((uint)*(byte *)(DAT_0000d420 + 0xd0) << 0x1d) < 0) {
    *(byte *)(DAT_0000d420 + 0xd0) = *(byte *)(DAT_0000d420 + 0xd0) & 0xfb;
    evt_flags_set(DAT_0000d410,0x400000);
  }
  return;
}



/* ======================================================================
 * 0000d15e  tx_ctx_free
 * ====================================================================== */

void tx_ctx_free(int param_1)

{
  int iVar1;
  
  iVar1 = DAT_0000d414;
  *(undefined4 *)(param_1 + 4) = *(undefined4 *)(DAT_0000d414 + 0x14);
  *(undefined2 *)(param_1 + 0x70) = 0xff;
  *(uint *)(param_1 + 0x80) = *(uint *)(param_1 + 0x80) | 0x20000;
  *(int *)(iVar1 + 0x14) = param_1;
  if (*(char *)(param_1 + 0x53) == '\0') {
    *(char *)(DAT_0000d40c + 5) = *(char *)(DAT_0000d40c + 5) + -1;
  }
  else {
    *(char *)(DAT_0000d40c + 4) = *(char *)(DAT_0000d40c + 4) + -1;
  }
  if (*(char *)(DAT_0000d420 + 0xb) != '\0') {
    tx_pending_ctrl_frame_dispatch();
  }
  if ((int)((uint)*(byte *)(DAT_0000d420 + 0xd0) << 0x1d) < 0) {
    *(byte *)(DAT_0000d420 + 0xd0) = *(byte *)(DAT_0000d420 + 0xd0) & 0xfb;
    evt_flags_set(DAT_0000d410,0x400000);
  }
  return;
}



/* ======================================================================
 * 0000d160  txq_set_frame_lifetime
 * ====================================================================== */

/* txq_set_frame_lifetime(pas, is_retry) -- set or extend a frame's expiry deadline.
   Takes the tx_ctx+0x54 view (xr_tx_pas).
   
     if (pas[4] == 0) {                       /* pas+0x10 == tx_ctx+0x64 */
         if ((hdr->addr1[0] & 1) == 0)        /* unicast */
             pas[4] = *(u32 *)(g_fw_ctx + if_id*0x98 + ac*4 + 0x4E8) << 10;
         else                                  /* group-addressed */
             pas[4] = 0x80000;
         return;
     }
     pas[4]  = fw_read_timer() + pas[4] * 0x400;   /* absolute deadline */
     pas[1] |= 0x20000;                            /* armed */
   
   *** CORRECTED: +0x4E8 is `max_rx_lifetime` from WSM 0x0013 SET_EDCA_PARAMS, and
   mainline programs 0xC8 = 200, NOT zero. ***
   An earlier revision of this comment identified +0x4E8 as `maxTransmitLifetime`
   from 0x0012 SET_TX_QUEUE_PARAMS and concluded that mainline's zero meant unicast
   frames never got a deadline.  Both halves were wrong.  edca_apply_params
   (0x000136A6) copies the 0x2C-byte 0x0013 payload to +0x4CC, and its layout puts
   max_rx_lifetime[4] at +0x4E8 (see that function's comment for the full map).
   cw1200 sets max_rx_lifetime = 0xC8 for every queue at sta.c:64-67 and again in
   cw1200_conf_tx (sta.c:639).
   
   So the lifetime is 200 << 10 = 204800 us ~ **205 ms**, the deadline IS armed, and
   frames DO expire.  0x0012's maxTransmitLifetime -- which mainline really does set
   to zero -- lands in the separate +0x400 table and is not read here.
   
   Group-addressed frames use the fixed 0x80000 (524288 us ~ 524 ms) regardless. */

void txq_set_frame_lifetime(int *param_1,int param_2)

{
  int iVar1;
  
  if (param_1[4] == 0) {
    if ((*(byte *)(*param_1 + 4) & 1) == 0) {
      iVar1 = *(int *)((uint)*(byte *)((int)param_1 + 0x69) * 0x98 + DAT_0000d424 +
                       (uint)*(byte *)(param_1 + 3) * 4 + 0x4e8) << 10;
    }
    else {
      iVar1 = 0x80000;
    }
    param_1[4] = iVar1;
    return;
  }
  if (-1 < param_1[1] << 0xe) {
    if (param_1[1] * 0x8000 < 0) {
      if (param_2 != 0) {
        return;
      }
    }
    else if (param_2 != 1) {
      return;
    }
    iVar1 = fw_read_timer();
    param_1[4] = iVar1 + param_1[4] * 0x400;
    param_1[1] = param_1[1] | 0x20000;
  }
  return;
}



/* ======================================================================
 * 0000d1c4  tx_frame_done_release
 * ====================================================================== */

void tx_frame_done_release(xr_tx_ctx *ctx)

{
  short sVar1;
  int iVar2;
  undefined1 *puVar3;
  
  sVar1 = *(short *)(DAT_0000d40c + 10) + 1;
  *(short *)(DAT_0000d40c + 10) = sVar1;
  if (sVar1 == 1) {
    phy_state_advance(0);
  }
  if ((byte)ctx->field_0xbd < 3) {
    iVar2 = (uint)(byte)ctx->field_0xbd * 0x3b0 + DAT_0000d428;
    *(short *)(iVar2 + 0x30) = *(short *)(iVar2 + 0x30) + 1;
  }
  puVar3 = &ctx->field_0x54;
  if ((*(int *)&ctx->field_0x58 * 4 < 0) && (0xd < (byte)ctx->field_0x63)) {
    bab_originate_addba(puVar3);
  }
  if (ctx->bCompletionClass == 9) {
    mac_set_ps_bit(ctx->field_0xbd,1);
  }
  else if ((byte)ctx->field_0xbd < 2) {
    ps_tx_done_followup(ctx);
  }
  if (*(ushort *)&ctx->field_0x70 < 0xfe) {
    tx_ctx_free_locked(puVar3);
    return;
  }
  *(uint *)&ctx->field_0x80 = *(uint *)&ctx->field_0x80 | 0x40;
  pas_retime_and_kick(puVar3);
  return;
}



/* ======================================================================
 * 0000d254  tx_complete_tala_adapt
 * ====================================================================== */

/* tx_complete_tala_adapt() -- TX completion drain + ADAPTIVE A-MPDU LENGTH
   CONTROL ("TALA", tx-ampdu-len-adaption -- the vendor header's comment on
   MIB 0x000B SET_TALA_PARA names it).
   
   *** This is the firmware mechanism that rewrites vif+0x128 (ampdu_num) at
   *** runtime.  MIB 0x000A SET_AMPDU_NUM only seeds a value this loop then
   *** overwrites.
   
   Parameters, both settable by MIB 0x000B SET_TALA_PARA (the handler at
   0xFFF0022C writes exactly these two words):
   
     g_tala_param0 @ 0x04001FC0 = 0x15020210
         bits  0..7  max ampdu_num              = 16
         bits  8..15 min ampdu_num (floor)      = 2
         bits 16..23 eval window / 200          = 2  -> 400 frames
         bits 24..27 good windows before +1     = 5
     g_tala_param1 @ 0x04001FC4 = 0x19140F0A   (four percent thresholds)
         bits 24..31 = 25  -> ampdu_num >>= 1        (halve)
         bits 16..23 = 20  -> ampdu_num  = 1/2 + 1/8 (x0.625)
         bits  8..15 = 15  -> ampdu_num  = 1/2 + 1/4 (x0.75)
         bits  0..7  = 10  -> ampdu_num  = 7/8       (x0.875)
   
   Per completed frame it accumulates, per link:
     ok/fail counts   at g_fw_ctx_d40c - 0x20 / - 0x18
     total tries      at            - 0x10   += tx_ctx[0x1E]
     penalty          at            - 0x08
         expected = __udivsi3(policy->short_retries * 15 + 99, 100);
         if (tries > expected) penalty += 1 << (tries - expected + 1);
         else                  penalty += tries;
     -- note the penalty is EXPONENTIAL in retries beyond `expected`.
   
   Every eval window (>= 400 frames, or penalty > 0x400) it compares
   short_retries*total against each threshold band and applies the matching
   multiplicative REDUCTION; if the window was good it bumps a counter and
   after 5 consecutive good windows grows ampdu_num by exactly +1.  Result is
   clamped to [min, max] and written back to
       vif + 0x128   (only when the per-link override at DAT_0000d708 is 0)
   then pushed to hardware via *(u32*)(DAT_0000d710 + 0x20).
   
   ASYMMETRY: one bad window can halve the aggregation (16 -> 8) instantly,
   but recovering 8 -> 16 needs 8 grow events = 40 good windows = 16000
   frames.  A 64 MiB upload is only ~46000 frames.  The state is per-vif and
   is NOT reset between transfers, so it persists across runs.  This is a
   mechanistic explanation for the recorded monotonic degradation across
   back-to-back runs (23 -> 25 -> 27 s) and the run-to-run hysteresis.
   
   DRIVER INTERACTION: `expected` is derived from the host's short_retries.
     mainline sends 7 -> expected = (105+99)/100 = 2
     both vendor trees send 6 -> expected = (90+99)/100 = 1
   So the vendor value penalises far more frames, predicting smaller
   aggregation and lower TX throughput -- consistent with the measured
   mainline-vs-vendor 1.5x TX advantage, and with candidate-tx-retry-policy.patch
   (which changes mainline to the vendor value) measuring WORSE.
   
   Neither mainline nor either vendor tree ever writes MIB 0x000B, so this
   runs entirely on the defaults above. */

void tx_complete_tala_adapt(void)

{
  short sVar1;
  ushort *puVar2;
  int *piVar3;
  byte bVar4;
  ushort uVar5;
  int iVar6;
  uint uVar7;
  undefined1 *puVar8;
  uint uVar9;
  int *piVar10;
  uint uVar11;
  uint uVar12;
  byte bVar13;
  uint uVar14;
  int iVar15;
  uint uVar16;
  uint uVar17;
  byte *pbVar18;
  uint uVar19;
  int *piVar20;
  int iVar21;
  undefined8 uVar22;
  uint local_3c;
  uint local_20;
  
  bVar13 = 0;
  uVar9 = *(uint *)(DAT_0000d40c + 0xc);
  uVar14 = *(uint *)(DAT_0000d40c + 0x10);
  if (*(int *)(DAT_0000d42c + 0x2c) != 0) {
    for (; uVar9 != uVar14; uVar9 = uVar9 + 1 & 0x3f) {
      if (*(char *)(*(int *)(uVar9 * 4 + DAT_0000d40c + 0x14) + -1) == '\0') {
        bVar13 = bVar13 + 1;
      }
    }
  }
  local_20 = *(uint *)(DAT_0000d40c + 0xc);
  do {
    iVar15 = DAT_0000d40c;
    if (local_20 == uVar14) {
      if ((*(short *)(DAT_0000d700 + 0x1a) == 0) &&
         (pac_phy_start_op(7), *(int *)(DAT_0000d718 + -0x6c) != 0)) {
        lmc_sched_radio_release();
      }
      return;
    }
    iVar6 = local_20 * 4 + DAT_0000d40c;
    piVar20 = *(int **)(iVar6 + 0x14);
    *(undefined4 *)(iVar6 + 0x14) = 0;
    local_20 = local_20 + 1 & 0x3f;
    *(uint *)(iVar15 + 0xc) = local_20;
    piVar10 = piVar20 + -0x15;
    sVar1 = (short)piVar20[7];
    uVar9 = (uint)*(byte *)((int)piVar20 + 0x69);
    if ((*(short *)(DAT_0000d430 + uVar9 * 2) == 0) ||
       (*(uint *)(uVar9 * 0x3b0 + DAT_0000d428 + 0x124) < 0x660)) {
      uVar11 = *DAT_0000d434;
      local_3c = (uVar11 & 0xffff) >> 8;
      if (sVar1 == 0xb) {
        iVar15 = DAT_0000d40c + -0x18;
LAB_0000d318:
        *(int *)(iVar15 + uVar9 * 4) = *(int *)(iVar15 + uVar9 * 4) + 1;
      }
      else if (sVar1 == 0) {
        iVar15 = DAT_0000d40c + -0x20;
        goto LAB_0000d318;
      }
      iVar6 = (uint)*(byte *)((int)piVar20 + 0x69) * 4;
      uVar9 = *(int *)(DAT_0000d40c + -0x18 + iVar6) + *(int *)(DAT_0000d40c + -0x20 + iVar6);
      uVar16 = (uint)*(byte *)((uint)*(byte *)((int)piVar20 + 0xe) * 0x14 + DAT_0000d424 + 0xf1);
      uVar7 = __udivsi3(uVar16 * 0xf + 99,100);
      iVar15 = DAT_0000d40c;
      *(uint *)(DAT_0000d40c + -0x10 + iVar6) =
           *(int *)(DAT_0000d40c + -0x10 + iVar6) + (uint)*(ushort *)((int)piVar20 + 0x1e);
      uVar12 = (uint)*(ushort *)((int)piVar20 + 0x1e);
      if ((uVar7 & 0xffff) < uVar12) {
        iVar6 = (uint)*(byte *)((int)piVar20 + 0x69) * 4;
        *(int *)(iVar15 + -8 + iVar6) =
             *(int *)(iVar15 + -8 + iVar6) + (1 << ((uVar12 - (uVar7 & 0xffff)) + 1 & 0xf));
      }
      else {
        iVar15 = (uint)*(byte *)((int)piVar20 + 0x69) * 4;
        *(uint *)(DAT_0000d40c + -8 + iVar15) = *(int *)(DAT_0000d40c + -8 + iVar15) + uVar12;
      }
      uVar7 = ((uVar11 & 0xffffff) >> 0x10) * 200;
      if ((uVar7 < uVar9 || uVar7 - uVar9 == 0) ||
         (0x400 < *(uint *)(DAT_0000d40c + -8 + (uint)*(byte *)((int)piVar20 + 0x69) * 4))) {
        uVar7 = DAT_0000d434[1];
        uVar12 = (uint)*(byte *)((int)piVar20 + 0x69);
        iVar15 = uVar12 * 4;
        iVar6 = *(int *)(DAT_0000d40c + -8 + iVar15);
        uVar17 = iVar6 * 100;
        uVar19 = *(ushort *)(uVar12 * 0x3b0 + DAT_0000d428 + 0x128) & 0xff;
        iVar21 = uVar16 * uVar9;
        uVar9 = iVar21 * (uVar7 >> 0x18);
        if (uVar9 < uVar17 || uVar9 + iVar6 * -100 == 0) {
          uVar19 = uVar19 >> 1;
          local_3c = 0;
        }
        else {
          uVar9 = iVar21 * ((uVar7 & 0xffffff) >> 0x10);
          if (uVar9 < uVar17 || uVar9 + iVar6 * -100 == 0) {
            uVar9 = uVar19 >> 3;
          }
          else {
            uVar9 = iVar21 * ((uVar7 & 0xffff) >> 8);
            if (uVar17 <= uVar9 && uVar9 + iVar6 * -100 != 0) {
              uVar9 = iVar21 * (uVar7 & 0xff);
              if (uVar9 < uVar17 || uVar9 + iVar6 * -100 == 0) {
                uVar19 = (uVar19 >> 1) + (uVar19 >> 2) + (uVar19 >> 3) & 0xff;
              }
              goto LAB_0000d482;
            }
            uVar9 = uVar19 >> 2;
          }
          uVar19 = (uVar19 >> 1) + uVar9;
          local_3c = 1;
        }
LAB_0000d482:
        if (*DAT_0000d438 + *DAT_0000d43c == 0) {
          if (((*DAT_0000d6f4 != '\0') && (*DAT_0000d6f0 < 0x32)) &&
             ((uint)((*(int *)(DAT_0000d700 + iVar15) + *(int *)(DAT_0000d700 + -0x10 + iVar15)) *
                    0x50) < (uint)(*(int *)(DAT_0000d700 + iVar15) * 100))) {
            *DAT_0000d6f4 = '\0';
          }
        }
        else {
          iVar15 = __udivsi3(*DAT_0000d438 * 600);
          uVar5 = __udivsi3(iVar15 + (uint)*DAT_0000d6f0 * 4,10);
          *DAT_0000d6f0 = uVar5;
          if (0x4b < uVar5) {
            *DAT_0000d6f4 = '\x01';
          }
          *DAT_0000d6f8 = 0;
          *DAT_0000d6fc = 0;
        }
        pbVar18 = DAT_0000d704;
        if (iVar21 * (uVar7 & 0xff) >> 1 < uVar17) {
          *(undefined1 *)(DAT_0000d700 + -0x14 + uVar12) = 0;
          bVar4 = *pbVar18;
          if (-1 < (int)((uint)bVar4 << 0x1e)) {
LAB_0000d52c:
            *pbVar18 = bVar4 | 1;
          }
        }
        else {
          bVar4 = *(char *)(DAT_0000d700 + -0x14 + uVar12) + 1;
          *(byte *)(DAT_0000d700 + -0x14 + uVar12) = bVar4;
          puVar2 = DAT_0000d6f0;
          if ((uVar11 & 0xfffffff) >> 0x18 <= (uint)bVar4) {
            *DAT_0000d6f4 = '\x01';
            uVar5 = *puVar2;
            *puVar2 = uVar5 - (uVar5 >> 5);
            uVar19 = uVar19 + 1 & 0xff;
            bVar4 = *DAT_0000d704;
            pbVar18 = DAT_0000d704;
            if ((int)((uint)bVar4 << 0x1e) < 0) goto LAB_0000d52c;
          }
        }
        if (*(short *)(DAT_0000d708 + (uint)*(byte *)((int)piVar20 + 0x69) * 2) == 0) {
          if ((local_3c <= uVar19) && (local_3c._0_2_ = (short)uVar19, (uVar11 & 0xff) < uVar19)) {
            local_3c._0_2_ = (short)(uVar11 & 0xff);
          }
          *(undefined2 *)((uint)*(byte *)((int)piVar20 + 0x69) * 0x3b0 + DAT_0000d70c + 0x128) =
               (undefined2)local_3c;
        }
        iVar15 = DAT_0000d700;
        *(undefined4 *)(DAT_0000d700 + -0x10 + (uint)*(byte *)((int)piVar20 + 0x69) * 4) = 0;
        *(undefined4 *)(iVar15 + -8 + (uint)*(byte *)((int)piVar20 + 0x69) * 4) = 0;
        *(undefined4 *)(iVar15 + 8 + (uint)*(byte *)((int)piVar20 + 0x69) * 4) = 0;
        iVar6 = DAT_0000d70c;
        *(undefined4 *)(iVar15 + (uint)*(byte *)((int)piVar20 + 0x69) * 4) = 0;
        *(uint *)(DAT_0000d710 + 0x20) =
             (uint)*(ushort *)((uint)*(byte *)((int)piVar20 + 0x69) * 0x3b0 + iVar6 + 0x128);
      }
    }
    *(short *)((int)piVar20 + -0x2e) = (short)piVar20[0x14];
    piVar3 = DAT_0000d714;
    if (piVar20[1] << 0x1a < 0) {
      uVar22 = u64_add_u32(DAT_0000d714[2],DAT_0000d714[3],(short)piVar20[2]);
      *(undefined8 *)(piVar3 + 2) = uVar22;
      piVar3[1] = piVar3[1] + 1;
      if (((piVar20[1] << 0x19 < 0) &&
          (*piVar3 = *piVar3 + 1, (piVar20[1] & 0x3fffffU) >> 0x14 != 0)) &&
         (((*(byte *)(DAT_0000d718 + 0x10) & 1) != 0 &&
          (puVar8 = (undefined1 *)lmc_msg_alloc(), puVar8 != (undefined1 *)0x0)))) {
        *puVar8 = 7;
        puVar8[0x28] = *(undefined1 *)((int)piVar20 + 0x69);
        puVar8[4] = *(undefined1 *)((int)piVar20 + 0x52);
        puVar8[0x29] = (char)piVar20[0x1b];
        puVar8[5] = *(undefined1 *)(DAT_0000d71c + (uint)*(byte *)(piVar20 + 3));
        *(short *)(puVar8 + 6) = (short)piVar20[0x15] << 4;
        *(undefined2 *)(puVar8 + 8) = *(undefined2 *)(*piVar20 + 4);
        *(undefined2 *)(puVar8 + 10) = *(undefined2 *)(*piVar20 + 6);
        *(undefined2 *)(puVar8 + 0xc) = *(undefined2 *)(*piVar20 + 8);
        evt_flags_set(DAT_0000d720,0x400000);
      }
    }
    uVar9 = (uint)*(byte *)((int)piVar20 + 0x69);
    if ((sVar1 == 0xb) && (*(char *)(uVar9 * 0x98 + DAT_0000d724 + 0x472) == '\x02')) {
      bab_mark_session_state5(piVar10);
    }
    *(short *)(DAT_0000d700 + 0x1a) = *(short *)(DAT_0000d700 + 0x1a) + -1;
    if (uVar9 < 3) {
      iVar15 = uVar9 * 0x3b0 + DAT_0000d70c;
      *(short *)(iVar15 + 0x30) = *(short *)(iVar15 + 0x30) + -1;
    }
    if (uVar9 < 2) {
      ps_timer_followup(piVar10);
    }
    if ((1 < bVar13) && (*(char *)((int)piVar20 + -1) == '\0')) {
      *(ushort *)((int)piVar20 + -0x2e) = *(ushort *)((int)piVar20 + -0x2e) | 0x20;
      bVar13 = bVar13 - 1;
    }
    piVar20[0xb] = piVar20[0xb] | 0x8000;
    tx_frame_complete(piVar10,sVar1);
  } while( true );
}



/* ======================================================================
 * 0000d728  ps_on_tx_complete
 * ====================================================================== */

void ps_on_tx_complete(int param_1,uint param_2)

{
  ushort uVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  bool bVar6;
  char cVar7;
  undefined8 uVar8;
  
  uVar5 = (uint)*(byte *)(param_1 + 0x17);
  iVar2 = uVar5 * 0x104 + DAT_0000db10;
  *(undefined2 *)(iVar2 + 0x134) = 0;
  if ((*(short *)(param_1 + 0x12) == 0x88) || (*(short *)(param_1 + 0x12) == 200)) {
    uVar3 = qos_dispatch(param_2 & 0xf);
  }
  else {
    uVar3 = 1;
  }
  bVar6 = ((uint)*(ushort *)(iVar2 + 0x5a) & 0x100 << (uVar3 & 0xff)) != 0;
  if (!bVar6) {
    *(undefined1 *)(iVar2 + 0xfd) = 0;
    *(ushort *)(iVar2 + 0x44) = *(ushort *)(iVar2 + 0x44) & 0xffaf;
  }
  iVar4 = DAT_0000db10;
  if (*(char *)(iVar2 + 0x40) != '\x01') {
    if ((*(char *)(iVar2 + 0x40) == '\0') && (*(char *)(iVar2 + 0xfc) != '\0')) {
      if (bVar6) {
        if ((int)((uint)*(ushort *)(iVar2 + 0x5a) << 0x19) < 0) {
          *(char *)(iVar2 + 0x53) = *(char *)(iVar2 + 0x53) + '\x01';
          uapsd_timer_restart(uVar5);
        }
        else {
          *(undefined1 *)(iVar2 + 0x42) = 0;
          timer_cancel(iVar2 + 0x98);
        }
      }
      else {
        *(ushort *)(iVar2 + 0x44) = *(ushort *)(iVar2 + 0x44) | 8;
        timer_start(iVar2 + 0xe8,*(undefined4 *)(iVar2 + 0x118));
      }
    }
    if (*(byte *)(iVar4 + 0x2a) < 3) {
      return;
    }
    goto LAB_0000d7da;
  }
  if ((*(ushort *)(param_1 + 0x10) & 0x48) == 8) {
    if (bVar6) {
      *(char *)(iVar2 + 0x53) = *(char *)(iVar2 + 0x53) + '\x01';
LAB_0000d808:
      *(undefined1 *)(iVar2 + 0x42) = 0;
      if (-1 < (int)((uint)*(ushort *)(iVar2 + 0x5a) << 0x19)) {
        timer_cancel(iVar2 + 0x98);
      }
      if ((((-1 < (int)((uint)*(ushort *)(iVar2 + 0x5a) << 0x1b)) &&
           ((~(uint)*(ushort *)(param_1 + 0x10) & 0x88) == 0)) && ((int)(param_2 << 0x1b) < 0)) &&
         ((int)((uint)*(ushort *)(param_1 + 0x10) << 0x12) < 0)) {
        if (-1 < *DAT_0000db14 << 0x18) {
          tx_send_qos_null(uVar5,uVar3);
          return;
        }
        *(ushort *)(iVar2 + 0x44) = *(ushort *)(iVar2 + 0x44) & 0xfffe;
        *(ushort *)(iVar2 + 0x46) = *(ushort *)(iVar2 + 0x46) | 1;
LAB_0000d7da:
        ps_try_enter_sleep_all();
        return;
      }
    }
    else {
      *(short *)(iVar2 + 0x10e) = *(short *)(iVar2 + 0x10e) + 1;
    }
  }
  else if (bVar6) goto LAB_0000d808;
  timer_cancel(iVar2 + 0xc0);
  if ((int)((uint)*(ushort *)(param_1 + 0x10) << 0x12) < 0) {
    iVar4 = uVar5 * 0x3b0 + DAT_0000db1c;
    if (*(char *)(uVar5 * 0x98 + DAT_0000db20 + 0x493) != '\0') {
      uVar8 = tsf_read(*(undefined1 *)(param_1 + 0x17));
      cVar7 = 0xfffffffe < iVar4 + 0x117U;
      u64_cmp((int)uVar8,(int)((ulonglong)uVar8 >> 0x20),*(undefined4 *)(iVar4 + 0x178),
              *(undefined4 *)(iVar4 + 0x17c));
      if (cVar7 != '\0') {
        uVar1 = *(ushort *)(iVar2 + 0x44);
        *(ushort *)(iVar2 + 0x44) = uVar1 & 0xffbf;
        if ((int)((uint)*(ushort *)(iVar2 + 0x5a) << 0x1b) < 0) {
          *(ushort *)(iVar2 + 0x44) = uVar1 & 0xffbe;
        }
        if ((int)((uint)*(ushort *)(iVar2 + 0x5a) << 0x19) < 0) {
          uapsd_timer_restart(uVar5);
        }
        ps_release_radio_if_all_idle(uVar5);
        goto LAB_0000d954;
      }
    }
    if (!bVar6) {
      if (*DAT_0000db14 << 0x18 < 0) {
        *(ushort *)(iVar2 + 0x44) = *(ushort *)(iVar2 + 0x44) & 0xffbf;
        *(ushort *)(iVar2 + 0x46) = *(ushort *)(iVar2 + 0x46) | 0x40;
      }
      else {
        *(undefined1 *)(iVar2 + 0x114) = 0;
        tx_send_ps_poll(uVar5);
      }
    }
  }
  else {
    uVar1 = *(ushort *)(iVar2 + 0x44);
    if (bVar6) {
      *(ushort *)(iVar2 + 0x44) = uVar1 & 0xfffe;
      if ((int)((uint)*(ushort *)(iVar2 + 0x5a) << 0x19) < 0) {
        uapsd_timer_restart(uVar5);
      }
    }
    else {
      if ((int)((uint)uVar1 << 0x19) < 0) {
        *(ushort *)(iVar2 + 0x44) = uVar1 & 0xffbf;
      }
      else if (((int)((uint)uVar1 << 0x17) < 0) && (*(char *)(iVar4 + 0x38) == '\0')) {
        iVar4 = fw_read_timer();
        if (DAT_0000db18 < iVar4 - *(int *)(iVar2 + 0x140)) {
          event_send_ps_mode_error(uVar5,3);
        }
        else {
          timer_start(iVar2 + 0xac,*(undefined4 *)(iVar2 + 0x120));
        }
      }
      if (*(int *)(iVar2 + 0x54) != 0) {
        ps_backoff_interval_update(uVar5);
      }
    }
  }
LAB_0000d954:
  ps_release_radio_if_all_idle(uVar5);
  return;
}



/* ======================================================================
 * 0000d95c  ps_tx_done_followup
 * ====================================================================== */

void ps_tx_done_followup(int param_1,undefined4 param_2,int param_3)

{
  byte bVar1;
  uint uVar2;
  uint uVar3;
  undefined4 uVar4;
  int iVar5;
  char *pcVar6;
  uint uVar7;
  
  bVar1 = *(byte *)(param_1 + 0xbd);
  iVar5 = (uint)bVar1 * 0x104 + DAT_0000db10;
  pcVar6 = (char *)(iVar5 + 0x40);
  uVar7 = (uint)*(ushort *)(param_1 + 0x5e);
  uVar2 = uVar7 & 0xff;
  if ((*(ushort *)(param_1 + 0x5e) & 0xf) == 0) {
    if ((uVar2 == 0xd0) || ((*pcVar6 == '\0' && (*(char *)(iVar5 + 0xfc) == '\0'))))
    goto LAB_0000da38;
    uVar4 = 0;
    if (uVar2 != 0x40) {
      vif_reset_all_state();
      goto LAB_0000da38;
    }
  }
  else {
    param_3 = *(int *)(param_1 + 0x58);
    if (param_3 << 6 < 0) {
      *(ushort *)(iVar5 + 0x44) = *(ushort *)(iVar5 + 0x44) & 0xffef | 0x40;
      goto LAB_0000da38;
    }
    if (-1 < (int)(uVar7 << 0x1c)) goto LAB_0000da38;
    uVar3 = (uint)*(ushort *)(iVar5 + 0x5a);
    uVar2 = 0;
    if ((uVar3 & 1 << *(sbyte *)(param_1 + 0x60)) != 0) {
      if ((int)(uVar3 << 0x1b) < 0) {
        if (((int)(uVar7 << 0x19) < 0) && (-1 < (int)(uVar3 << 0x1a))) {
          *(undefined2 *)(param_1 + 0x70) = 0;
LAB_0000d9f4:
          uVar2 = 0x1000000;
        }
      }
      else if (param_3 << 9 < 0) {
        *(ushort *)(iVar5 + 0x44) = *(ushort *)(iVar5 + 0x44) | 1;
        goto LAB_0000d9f4;
      }
    }
    uVar7 = *(uint *)(param_1 + 0x58) | uVar2;
    *(uint *)(param_1 + 0x58) = uVar7;
    if (((((*(char *)(iVar5 + 0xfc) == '\0') || (*pcVar6 != '\x01')) || (uVar2 != 0)) ||
        (((int)((uint)*(ushort *)(param_1 + 0x5e) << 0x19) < 0 || ((int)(uVar7 << 1) < 0)))) ||
       (2 < *(byte *)(DAT_0000db10 + 0x2a))) goto LAB_0000da38;
    uVar4 = 0x100;
    *(ushort *)(iVar5 + 0x44) = *(ushort *)(iVar5 + 0x44) & 0xfeff;
    uVar2 = 0;
    *pcVar6 = '\0';
  }
  mac_set_ps_bit((uint)bVar1,0,uVar2,uVar4,param_2,param_3);
LAB_0000da38:
  *(int *)(iVar5 + 0x5c) = *(int *)(iVar5 + 0x5c) + 1;
  return;
}



/* ======================================================================
 * 0000da40  ps_timer_followup
 * ====================================================================== */

void ps_timer_followup(int param_1)

{
  uint uVar1;
  int iVar2;
  int iVar3;
  
  if (*(short *)(param_1 + 0x70) == 0x16) {
    return;
  }
  uVar1 = (uint)*(byte *)(param_1 + 0xbd);
  iVar3 = uVar1 * 0x104 + DAT_0000db10;
  if (((*(char *)(iVar3 + 0xfc) != '\0') && ((*(uint *)(param_1 + 0x58) & 0x3ffffff) >> 0x18 == 0))
     && ((*(ushort *)(param_1 + 0x5e) & 0x4f) != 0x48)) {
    *(ushort *)(iVar3 + 0x44) = *(ushort *)(iVar3 + 0x44) | 8;
    timer_start(iVar3 + 0xe8,*(undefined4 *)(iVar3 + 0x118));
  }
  iVar2 = DAT_0000db10;
  if (*(int *)(DAT_0000db24 + 0x28) != 0) {
    if (*(byte *)(DAT_0000db10 + 0x2a) < 3) {
      return;
    }
    timer_start(iVar3 + 0xe8,&DAT_00001f40);
    if (*(char *)(iVar2 + 0x2a) != '\x04') {
      return;
    }
LAB_0000db92:
    ps_try_enter_sleep_all();
    return;
  }
  if (*(short *)(param_1 + 0x70) == 0) {
    *(undefined2 *)(uVar1 * 0x3b0 + DAT_0000db1c + 0x1e2) = 0;
  }
  if (*(char *)(iVar3 + 0x40) != '\x01') {
    if (*(char *)(iVar3 + 0x40) != '\0') {
      return;
    }
    if (*(byte *)(iVar2 + 0x2a) < 3) {
      return;
    }
    goto LAB_0000db92;
  }
  if ((*(ushort *)(iVar3 + 0x136) != 0) && (*(ushort *)(iVar3 + 0x136) < *(ushort *)(iVar3 + 0x134))
     ) {
    *(undefined2 *)(iVar3 + 0x134) = 0;
    *(ushort *)(iVar3 + 0x44) = *(ushort *)(iVar3 + 0x44) | 4;
    timer_start(iVar3 + 0xac,*(undefined4 *)(iVar3 + 0x120));
  }
  if (*(int *)(param_1 + 0x58) << 6 < 0) {
    timer_start(iVar3 + 0xc0,*(undefined4 *)(DAT_0000db10 + 0x30));
    return;
  }
  if (((uint)*(ushort *)(iVar3 + 0x5a) & 1 << (uint)*(byte *)(param_1 + 0x60)) == 0) {
    if ((*(uint *)(iVar3 + 0x54) == 0) ||
       (iVar2 = *(int *)(iVar3 + 0x128), *(uint *)(iVar3 + 0x54) < (uint)(iVar2 << 1)))
    goto LAB_0000db80;
    iVar3 = iVar3 + 0x70;
  }
  else {
    if ((*(ushort *)(iVar3 + 0x5a) & 0x3f) >> 4 == 1) {
      if ((*(ushort *)(iVar3 + 0x44) & 1) == 0) {
        *(ushort *)(iVar3 + 0x44) = *(ushort *)(iVar3 + 0x44) | 1;
        ps_send_pending_poll_or_qosnull(uVar1);
      }
    }
    else if (*(int *)(param_1 + 0x58) << 7 < 0) {
      *(ushort *)(iVar3 + 0x44) = *(ushort *)(iVar3 + 0x44) | 1;
    }
    if ((int)((uint)*(ushort *)(iVar3 + 0x5a) << 0x19) < 0) goto LAB_0000db80;
    iVar3 = iVar3 + 0x98;
    iVar2 = DAT_0000db98;
  }
  timer_start(iVar3,iVar2);
LAB_0000db80:
  ps_release_radio_if_all_idle(uVar1);
  return;
}



/* ======================================================================
 * 0000db9c  rx_dup_cache_check
 * ====================================================================== */

undefined4 rx_dup_cache_check(uint param_1,short *param_2,int param_3,uint param_4)

{
  short *psVar1;
  uint uVar2;
  uint uVar3;
  
  uVar2 = *(uint *)(DAT_0000dc30 + 0xc);
  if ((param_1 & 0xf) == 0) {
    param_4 = (param_4 & 0xffffff00) + 0x12;
  }
  if (((param_3 != 0) || ((param_1 & 0xff) != 0x48)) &&
     (uVar3 = uVar2, (param_1 & DAT_0000dc34) != 0xc0)) {
    do {
      psVar1 = (short *)(uVar3 * 0xc + DAT_0000dc38 + DAT_0000dc3c);
      if ((((ushort)psVar1[3] == param_4) && (*(int *)(psVar1 + 4) == param_3)) &&
         ((*param_2 == *psVar1 && ((param_2[1] == psVar1[1] && (param_2[2] == psVar1[2])))))) {
        return 1;
      }
      uVar3 = uVar3 - 1 & 0x1f;
    } while (uVar2 != uVar3);
    uVar2 = uVar2 + 1 & 0x1f;
    *(uint *)(DAT_0000dc30 + 0xc) = uVar2;
    psVar1 = (short *)(uVar2 * 0xc + DAT_0000dc38 + DAT_0000dc3c);
    *psVar1 = *param_2;
    psVar1[1] = param_2[1];
    psVar1[2] = param_2[2];
    *(int *)(psVar1 + 4) = param_3;
    psVar1[3] = (short)param_4;
  }
  return 0;
}



/* ======================================================================
 * 0000dc40  txq_list_insert
 * ====================================================================== */

void txq_list_insert(int param_1,undefined4 param_2,int param_3)

{
  int *piVar1;
  
  piVar1 = DAT_0000dcb0;
  if (param_3 == 0) {
    *(undefined4 *)(param_1 + 4) = 0;
    if (piVar1[1] == 0) {
      *piVar1 = param_1;
    }
    else {
      *(int *)(piVar1[1] + 4) = param_1;
    }
    piVar1[1] = param_1;
  }
  else if (param_3 == 2) {
    *(int *)(param_1 + 4) = *DAT_0000dcb0;
    if (piVar1[1] == 0) {
      piVar1[1] = param_1;
    }
    *piVar1 = param_1;
    return;
  }
  return;
}



/* ======================================================================
 * 0000dc6c  txq_list_remove
 * ====================================================================== */

void txq_list_remove(int param_1,int *param_2)

{
  int *piVar1;
  int iVar2;
  int iVar3;
  
  piVar1 = DAT_0000dcb0;
  if (*DAT_0000dcb0 == param_1) {
    iVar3 = *(int *)(param_1 + 4);
    *DAT_0000dcb0 = iVar3;
  }
  else {
    iVar2 = *DAT_0000dcb0;
    if (*(int *)(*param_2 + 4) == param_1) {
      *(undefined4 *)(*param_2 + 4) = *(undefined4 *)(param_1 + 4);
      goto LAB_0000dc9e;
    }
    do {
      iVar3 = iVar2;
      if (iVar3 == 0) goto LAB_0000dc9e;
      iVar2 = *(int *)(iVar3 + 4);
    } while (*(int *)(iVar3 + 4) != param_1);
    *(undefined4 *)(iVar3 + 4) = *(undefined4 *)(param_1 + 4);
  }
  *param_2 = iVar3;
LAB_0000dc9e:
  if (piVar1[1] == param_1) {
    piVar1[1] = *param_2;
  }
  *(undefined4 *)(param_1 + 4) = 0;
  return;
}



/* ======================================================================
 * 0000dd58  enc_que_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000dd78) */
/* WARNING: Removing unreachable block (ram,0x0000dd78) */

void enc_que_dispatch(int param_1)

{
  uint uVar1;
  
  if (9 < *(byte *)(param_1 + 0xca)) {
    fw_assert(s_pGtx_enc_que_07_c_0000dea9 + 3,0x59,0x14);
  }
  uVar1 = (uint)*(byte *)(param_1 + 0xca);
                    /* WARNING: Could not recover jumptable at 0x0000dd78. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0000dd7c <= uVar1) {
    uVar1 = (uint)DAT_0000dd7c;
  }
  (*(code *)((uint)*(byte *)(uVar1 + 0xdd7d) * 2 + 0xdd7d))();
  return;
}



/* ======================================================================
 * 0000de2c  tkip_next_iv_skip_weak
 * ====================================================================== */

uint tkip_next_iv_skip_weak(uint param_1,uint param_2)

{
  uint uVar1;
  uint uVar2;
  int iVar3;
  uint uVar4;
  undefined4 local_1c;
  
  local_1c = param_1;
  do {
    while( true ) {
      local_1c = local_1c + 1;
      if (0xffffff < local_1c) {
        local_1c = 0;
      }
      uVar1 = local_1c & 0xff;
      uVar2 = local_1c >> 8 & 0xff;
      uVar4 = uVar1 + uVar2 & 0xff;
      if (param_2 < uVar4) break;
      iVar3 = param_2 - uVar4;
      if (0xff - iVar3 < (int)uVar1) {
        local_1c = CONCAT31(local_1c._1_3_,0xff);
      }
      else {
        local_1c = CONCAT31(local_1c._1_3_,(char)local_1c + (char)iVar3);
      }
    }
  } while (((((uVar2 == 0xff) && (uVar1 < param_2 + 3)) && (2 < uVar1)) ||
           (((uVar1 == 1 && (uVar2 <= param_2 >> 1)) && (uVar2 != 0)))) ||
          (((0xff - uVar1 == uVar2 && (uVar1 <= (param_2 >> 1) + 1)) && (1 < uVar1))));
  return local_1c;
}



/* ======================================================================
 * 0000dec4  tx_lmac_req_submit
 * ====================================================================== */

undefined4 tx_lmac_req_submit(xr_tx_ctx *param_1)

{
  ushort uVar1;
  int iVar2;
  undefined4 uVar3;
  uint uVar4;
  undefined1 *puVar5;
  int iVar6;
  int iVar7;
  
  iVar6 = DAT_0000e084;
  puVar5 = &param_1->field_0x80;
  *(undefined4 *)puVar5 = 1;
  if (2 < *(byte *)(iVar6 + 10)) {
    fw_assert(s_tx_lmac_req_02_c_0000e088,0x4b,1000);
  }
  param_1->field_0xbd = *(undefined1 *)(iVar6 + 10);
  iVar2 = DAT_0000e0a0;
  if (*(uint *)(*(int *)param_1 + 0xc) >> 8 == DAT_0000e09c) {
    param_1->field_0xbd = *(undefined1 *)(DAT_0000e0a0 + 0x16);
  }
  param_1->field_0xbf = (char)((param_1->bQueueId & 0x3f) >> 2);
  param_1->bQueueId = param_1->bQueueId & 3;
  uVar3 = fw_read_timer();
  *(undefined4 *)&param_1->field_0x40 = uVar3;
  param_1->bCompletionClass = 0;
  param_1->wField50 = 0;
  param_1->bField52 = 1;
  *(undefined4 *)&param_1->field_0x4c = 0;
  *(undefined4 *)&param_1->field_0x58 = 0;
  *(undefined4 *)&param_1->field_0x74 = 0;
  *(undefined4 *)&param_1->field_0x78 = 0;
  *(undefined4 *)&param_1->field_0x7c = 0;
  iVar7 = *(int *)&param_1->field_0x40 + -1;
  *(int *)&param_1->field_0x68 = iVar7;
  *(undefined4 *)&param_1->field_0x38 = 0;
  *(undefined4 *)&param_1->field_0x3c = 0;
  *(int *)&param_1->field_0x6c = iVar7;
  iVar6 = (uint)(byte)param_1->field_0xbd * 0x3b0 + iVar6;
  if ((1 << (uint)(byte)param_1->field_0xbf & (uint)*(ushort *)(iVar6 + 0x2c)) == 0) {
    *(undefined2 *)&param_1->field_0x72 = 0;
    uVar3 = 0x14;
  }
  else {
    if ((*(uint *)&param_1->field_0x14 & 3) == 1) {
      *(undefined4 *)&param_1->field_0x58 = 8;
    }
    *(uint *)&param_1->field_0x58 = *(uint *)&param_1->field_0x58 | 0x800000;
    *(undefined4 *)&param_1->field_0x64 = *(undefined4 *)&param_1->field_0x10;
    if ((param_1->bAllocFlags & 1) != 0) {
      *(uint *)&param_1->field_0x58 = *(uint *)&param_1->field_0x58 | 0x10000;
    }
    iVar7 = DAT_0000e0a4;
    param_1->field_0x61 = (char)((param_1->bAllocFlags & 0xf) >> 1);
    *(uint *)&param_1->field_0x54 = param_1->dwHdr80211;
    param_1->field_0x60 = *(undefined1 *)(iVar7 + (uint)param_1->bQueueId);
    txq_set_frame_lifetime(&param_1->field_0x54,0);
    *(uint *)&param_1->field_0x58 =
         *(int *)&param_1->field_0x58 + (*(uint *)&param_1->field_0x14 >> 0xb & 0xe0);
    *(short *)&param_1->field_0x5c = (short)*(undefined4 *)&param_1->field_0x18;
    *(undefined2 *)&param_1->field_0x70 = 0xfe;
    *(undefined2 *)&param_1->field_0x72 = 0;
    *(undefined2 *)&param_1->field_0xa4 = 0;
    param_1->field_0xa7 = param_1->bField52;
    *(undefined4 *)&param_1->field_0x90 = 0;
    param_1->field_0x63 = param_1->field_0xc;
    param_1->field_0x62 = (char)((param_1->bAllocFlags & 0x7f) >> 4);
    if (((int)(*(uint *)(iVar6 + 0x1c) << 0x1d) < 0) &&
       ((uVar4 = param_1->dwHdr80211, (*(byte *)(uVar4 + 4) & 1) != 0 ||
        (((*(ushort *)(uVar4 + 4) == DAT_0000e0a8 && (*(ushort *)(uVar4 + 6) == DAT_0000e0a8)) &&
         (*(ushort *)(uVar4 + 8) == DAT_0000e0a8)))))) {
      if ((*(uint *)(iVar6 + 0x1c) & 7) >> 1 == 3) {
        *(char *)(iVar6 + 0x163) = *(char *)(iVar6 + 0x163) + '\x01';
      }
      iVar6 = DAT_0000e0a0;
      uVar1 = *(ushort *)(iVar2 + 4);
      if ((uVar1 & 1) == 0) {
        *(undefined1 *)(iVar2 + 0x10) = param_1->field_0x24;
        *(byte *)(iVar2 + 0x11) = param_1->bAllocFlags & 0x7f;
        *(undefined4 *)(iVar6 + 8) = *(undefined4 *)&param_1->field_0x10;
        *(undefined4 *)(iVar6 + 0xc) = *(undefined4 *)&param_1->field_0x14;
        *(ushort *)(iVar2 + 4) = uVar1 | 1;
      }
    }
    iVar6 = pas_tx_policy_prepare(&param_1->field_0x54);
    if (iVar6 == 1) {
      *(undefined4 *)&param_1->field_0xd8 = 0;
      *(undefined4 *)&param_1->field_0x130 = 0;
      *(undefined4 *)&param_1->field_0x4c = 0;
      tx_classify_hdr_len(param_1);
      *(uint *)puVar5 = *(uint *)puVar5 | 2;
      mic_build_aad_and_submit(param_1);
      return 0;
    }
    uVar3 = 0x17;
    *(undefined2 *)&param_1->field_0x72 = 0;
  }
  tx_frame_complete(param_1,uVar3);
  return 0;
}



/* ======================================================================
 * 0000e0f4  mic_build_aad_and_submit
 * ====================================================================== */

void mic_build_aad_and_submit(int param_1)

{
  byte bVar1;
  ushort uVar2;
  int iVar3;
  ushort *dst;
  undefined1 *puVar4;
  uint uVar5;
  void *src;
  void *src_00;
  void *src_01;
  uint uVar6;
  void *dst_00;
  
  bVar1 = 0;
  if ((*(char *)(param_1 + 0xca) != '\x03') && (*(char *)(param_1 + 0xca) != '\x02'))
  goto LAB_0000e1dc;
  iVar3 = *(int *)(param_1 + 0x44);
  src = *(void **)(param_1 + 0x1c);
  dst_00 = (void *)((int)src + iVar3 + -8);
  dst = (ushort *)(param_1 + 0xe8);
  fw_memcpy(dst,src,iVar3 + 8);
  uVar2 = *dst & 0x300;
  src_00 = (void *)(param_1 + 0xf2);
  if ((*dst & 0x300) == 0) {
    src_01 = (void *)(param_1 + 0xec);
LAB_0000e168:
    fw_memcpy(dst_00,src_01,6);
LAB_0000e172:
    fw_memcpy((void *)((int)src + iVar3 + -2),src_00,6);
  }
  else {
    src_01 = (void *)(param_1 + 0xf8);
    if (uVar2 == 0x100) goto LAB_0000e168;
    if (uVar2 == 0x200) {
      fw_memcpy(dst_00,(void *)(param_1 + 0xec),6);
      src_00 = src_01;
      goto LAB_0000e172;
    }
    if (uVar2 == 0x300) {
      fw_memcpy(dst_00,src_01,6);
      src_00 = (void *)(param_1 + 0x100);
      goto LAB_0000e172;
    }
  }
  if ((*dst & 0x8f) == 0x88) {
    bVar1 = (byte)*(undefined2 *)(param_1 + 0x100) & 0xf;
  }
  *(byte *)((int)src + iVar3 + 4) = bVar1;
  *(undefined1 *)((int)src + iVar3 + 5) = 0;
  *(undefined1 *)((int)src + iVar3 + 6) = 0;
  *(undefined1 *)((int)src + iVar3 + 7) = 0;
  *(void **)(param_1 + 0xd4) = dst_00;
  iVar3 = *(int *)(param_1 + 0x48);
  uVar6 = iVar3 + 4U >> 2;
  puVar4 = (undefined1 *)(*(int *)(param_1 + 0x1c) + (uint)*(ushort *)(param_1 + 0x5c) + -0xc);
  *puVar4 = 0x5a;
  for (uVar5 = 0; puVar4 = puVar4 + 1, uVar5 < (uVar6 * 4 - (iVar3 + -3) & 0xff);
      uVar5 = uVar5 + 1 & 0xff) {
    *puVar4 = 0;
  }
  *(uint *)(param_1 + 0xd8) = uVar6;
LAB_0000e1dc:
  *(undefined4 *)(param_1 + 0xe4) = DAT_0000e1ec;
  mic_submit_or_queue(param_1 + 0xd0);
  return;
}



/* ======================================================================
 * 0000e1f0  tx_select_key_and_cipher
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0000e292) */
/* WARNING: Removing unreachable block (ram,0x0000e292) */

void tx_select_key_and_cipher(int param_1)

{
  int iVar1;
  undefined1 uVar2;
  uint uVar3;
  int iVar4;
  ushort *puVar5;
  undefined2 local_28;
  
  puVar5 = *(ushort **)(param_1 + 0x1c);
  iVar4 = (uint)*(byte *)(param_1 + 0xbd) * 0x3b0 + DAT_0000e588;
  *(undefined1 *)(param_1 + 0xca) = 9;
  if (((*puVar5 & 0xff) == 0xb0) && (*(int *)(iVar4 + 0x138) << 0x1d < 0)) {
    return;
  }
  if (-1 < (int)((uint)*(ushort *)(param_1 + 0x5e) * 0x20000)) {
    if ((*(uint *)(iVar4 + 0x138) & 1) == 0) {
      return;
    }
    if ((puVar5[2] & 1) == 0) {
      return;
    }
    iVar1 = frame_is_unprotected_mgmt(puVar5);
    if (iVar1 == 0) {
      return;
    }
    local_28 = 4;
    iVar1 = key_lookup_for_frame(*(undefined1 *)(iVar4 + 0x1a),puVar5 + 2,4);
    if (iVar1 != 0) {
LAB_0000e38e:
      iVar4 = ie_find_in_mgmt_frame(puVar5,*(undefined4 *)(param_1 + 0x18),0x4c,0);
      *(int *)(param_1 + 0xc4) = iVar4;
      if (iVar4 == 0) {
        return;
      }
      *(undefined1 *)(param_1 + 0xca) = 8;
      *(int *)(param_1 + 0xcc) = iVar1;
      fw_memcpy((void *)(param_1 + 0x118),(void *)(iVar1 + 4),0x10);
      *(undefined2 *)(param_1 + 0x116) = 0x10;
      *(undefined2 *)(*(int *)(param_1 + 0xc4) + 2) = local_28;
      return;
    }
    local_28 = 5;
    iVar1 = key_lookup_for_frame(*(undefined1 *)(iVar4 + 0x1a),puVar5 + 2,5);
    if (iVar1 != 0) goto LAB_0000e38e;
    goto LAB_0000e344;
  }
  if (1 < *(byte *)(param_1 + 0xbd)) {
    fw_assert(s_tx_set_frm_03_c_0000e58c,0x119,1000);
  }
  uVar2 = 0xf;
  if ((puVar5[2] & 1) == 0) {
LAB_0000e258:
    iVar1 = key_lookup_for_frame(*(undefined1 *)(iVar4 + 0x1a),puVar5 + 2,uVar2);
    if (iVar1 != 0) goto LAB_0000e278;
  }
  else if (*(int *)(iVar4 + 0x1c) << 0x1d < 0) {
    uVar2 = *(undefined1 *)(iVar4 + 0x162);
    goto LAB_0000e258;
  }
  if ((*(char *)(iVar4 + 0x111) == -1) ||
     (iVar1 = key_lookup_for_frame(*(undefined1 *)(iVar4 + 0x1a),puVar5 + 2), iVar1 == 0)) {
LAB_0000e344:
    *(undefined2 *)(param_1 + 0x70) = 0x10;
    return;
  }
LAB_0000e278:
  *(int *)(param_1 + 0xcc) = iVar1;
  *(undefined1 *)(param_1 + 0xca) = *(undefined1 *)(iVar1 + 1);
  *(undefined4 *)(param_1 + 0x168) = 0;
  uVar3 = (uint)*(byte *)(iVar1 + 1);
                    /* WARNING: Could not recover jumptable at 0x0000e292. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0000e296 <= uVar3) {
    uVar3 = (uint)DAT_0000e296;
  }
  (*(code *)((uint)*(byte *)(uVar3 + 0xe297) * 2 + 0xe297))(param_1 + 0xd0);
  return;
}



/* ======================================================================
 * 0000e3c6  tx_classify_hdr_len
 * ====================================================================== */

/* tx_classify_hdr_len(tx_ctx) -- classify the 802.11 header and split lengths.
   Base is the OUTER tx_ctx (not the +0x54 pas view).
   
     hdr_len = 0x18 for 3-address, 0x1E for 4-address; QoS data (subtype 0x88)
               adds the 2-byte QoS control field.
     tx_ctx+0x44 = hdr_len
     tx_ctx+0x48 = *(u16 *)(tx_ctx + 0x5C) - hdr_len    /* PAYLOAD BYTES */
   
   *** DISAMBIGUATION: tx_ctx+0x48 here is a payload BYTE length.  It is NOT the
   airtime field.  The airtime lives at tx_ctx+0x9C and is reached as pas+0x48
   through the tx_ctx+0x54 sub-struct -- see pas_compute_tx_timing (0x7FA6).
   Docs written before this was noticed said "tx_ctx+0x48 = airtime"; the unit
   claim was right, the address was not. *** */

void tx_classify_hdr_len(xr_tx_ctx *ctx)

{
  ushort uVar1;
  ushort uVar2;
  ushort uVar3;
  uint uVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  int iVar8;
  uint uVar9;
  ushort *puVar10;
  ushort local_20;
  
  local_20 = 0;
  uVar3 = local_20;
  local_20 = 0;
  iVar8 = (uint)(byte)ctx->field_0xbd * 0x3b0 + DAT_0000e588;
  uVar4 = *(uint *)&ctx->field_0x58;
  uVar5 = uVar4 | 0x1000;
  *(uint *)&ctx->field_0x58 = uVar5;
  uVar9 = 0;
  if (ctx->bCompletionClass != 0) {
    if (ctx->bCompletionClass == 5) {
      uVar5 = uVar5 ^ 0x3000;
    }
    else {
      if (ctx->bField52 != 0) goto LAB_0000e418;
      uVar5 = uVar4 | 0x3000;
    }
    *(uint *)&ctx->field_0x58 = uVar5;
  }
LAB_0000e418:
  puVar10 = (ushort *)ctx->dwHdr80211;
  uVar1 = *puVar10;
  uVar4 = (uint)uVar1;
  *(ushort *)&ctx->field_0x5e = uVar1;
  if ((puVar10[2] & 1) != 0) {
    *(uint *)&ctx->field_0x58 = *(uint *)&ctx->field_0x58 | 0x300;
    *(int *)(iVar8 + 0x74) = *(int *)(iVar8 + 0x74) + 1;
  }
  if ((puVar10[8] & 1) != 0) {
    *(int *)(iVar8 + 0x74) = *(int *)(iVar8 + 0x74) + 1;
  }
  uVar2 = *(ushort *)&ctx->field_0x5c;
  if ((uVar4 & 0xf) == 4) {
    if ((uVar4 & 0xff) == 0x84) {
      *(uint *)&ctx->field_0x58 = *(uint *)&ctx->field_0x58 | 0x4000;
    }
    uVar6 = *(uint *)&ctx->field_0x58 | 1;
    uVar5 = (uint)uVar2;
    local_20 = uVar3;
  }
  else {
    uVar5 = uVar4 & 0x300;
    if ((((uVar1 & 0x300) == 0) || (uVar5 == 0x100)) || (uVar5 == 0x200)) {
      uVar9 = 0x18;
    }
    else if (uVar5 == 0x300) {
      uVar9 = 0x1e;
    }
    if ((uVar4 & 0x8f) != 0x88) {
      pas_latch_vif_slot_flag(&ctx->field_0x54);
      goto LAB_0000e514;
    }
    uVar5 = *(uint *)&ctx->field_0x58;
    *(uint *)&ctx->field_0x58 = uVar5 | 0x400000;
    ctx->field_0xa6 = (byte)*(undefined2 *)(ctx->dwHdr80211 + 0x18) & 0xf;
    if ((puVar10[2] & 1) == 0) {
      *(uint *)&ctx->field_0x58 = uVar5 | 0x400001;
      if (-1 < (int)(uVar4 << 0x19)) {
        *(uint *)&ctx->field_0x58 = uVar5 | 0x20400001;
        tx_assign_seq_num();
      }
    }
    else {
      pas_latch_vif_slot_flag(&ctx->field_0x54);
    }
    uVar5 = uVar9 + 2;
    if ((int)(uVar4 << 0x10) < 0) {
      uVar5 = uVar9 + 6;
    }
    if ((uVar4 & 0x3ff) >> 8 == 3) {
      local_20 = puVar10[0xf];
    }
    else {
      local_20 = puVar10[0xc];
    }
    uVar6 = ((local_20 & 0x7f) >> 5) << 0x14 | *(uint *)&ctx->field_0x58 & 0xffcfffff;
    *(uint *)&ctx->field_0x58 = uVar6;
    uVar7 = (local_20 & 0x7f) >> 5;
    if ((uVar7 != 1) && (uVar9 = uVar5, uVar7 != 3)) goto LAB_0000e514;
    uVar6 = uVar6 | 0x200;
  }
  *(uint *)&ctx->field_0x58 = uVar6;
  uVar9 = uVar5;
LAB_0000e514:
  *(ushort *)&ctx->field_0xc8 = local_20;
  ctx->dwHdrLen = uVar9;
  ctx->dwPayloadLen = uVar2 - uVar9;
  if ((byte)ctx->field_0xbd < 2) {
    iVar8 = (uint)(byte)ctx->field_0xbd * 0x3b0 + DAT_0000e588;
    if (((*(uint *)(iVar8 + 0x124) < *(ushort *)&ctx->field_0x5c + 4) &&
        (-1 < (int)(*(uint *)&ctx->field_0x58 << 0x17))) &&
       (((int)(uVar4 << 0x1c) < 0 && (*DAT_0000e59c == '\0')))) {
      *(uint *)&ctx->field_0x58 = *(uint *)&ctx->field_0x58 | 0x400;
    }
    if ((*(char *)(iVar8 + 0x26) != '\0') && (3 < (byte)ctx->field_0x63)) {
      *(uint *)&ctx->field_0x58 = *(uint *)&ctx->field_0x58 | 0x800;
    }
  }
  ctx->field_0xaa = 0xff;
  tx_select_key_and_cipher(ctx);
  return;
}



/* ======================================================================
 * 0000e5a0  wsm_dispatch_cmd
 * ====================================================================== */

/* WSM command dispatcher (hi_msg.c).
   
     if_id = (msgid >> 6) & 3;  if (if_id > 2) msgid = 0x0FFF   // only 3 VIFs
     idx   = msgid & 0x0C3F;    if (idx > 0x24) idx = 0x24
     jump  wsm_cmd_dispatch_table[idx]              // 0x04000710, 37 entries
   
   Index 0x24 and every unimplemented opcode point at wsm_cmd_unsupported
   (0x00016182), which replies msgid|0x0400 with MsgLen = 4, i.e. a confirm
   with NO status word.  A host that expects a 4-byte status (cw1200
   wsm_generic_confirm) sees a buffer underflow on such a reply.
   
   Table contents (opcode -> handler):
     0x00 0x16078   0x01 0x160e8   0x02 UNSUPPORTED  0x03 0x158cc
     0x04 0x0b600 tx_req (data)    0x05 read_mib     0x06 write_mib
     0x07 start_scan               0x08 stop_scan    0x09 configuration
     0x0A reset                    0x0B join         0x0C add_key
     0x0D remove_key               0x0E start_measure (11k, IMPLEMENTED)
     0x0F 0x10e4e unknown          0x10 set_pm       0x11 set_bss_params
     0x12 set_tx_queue_params      0x13 edca_params  0x14 0x06d30 unknown
     0x15 0x10eda unknown          0x16 switch_channel
     0x17 start                    0x18 beacon_transmit
     0x19 UNSUPPORTED  <- vendor driver's wsm_start_find
     0x1A UNSUPPORTED  <- vendor driver's wsm_stop_find
     0x1B update_ie                0x1C map_link     0x1D 0x10f76 unknown
     0x1E UNSUPPORTED  0x1F UNSUPPORTED
     0x20 0x10f96 unknown          0x21 UNSUPPORTED
     0x22 init_release_buffer      0x23 request_buffer (IMPLEMENTED)
     0x24 UNSUPPORTED (catch-all)
   
   Message header is { u16 MsgLen; u16 MsgId }.  MsgId bits [15:13] carry the
   host TX sequence number and are checked/stripped in hif_rx_process. */

void wsm_dispatch_cmd(int param_1)

{
  ushort uVar1;
  uint uVar2;
  uint uVar3;
  
  uVar1 = *(ushort *)(param_1 + 2);
  uVar3 = (uVar1 & 0xff) >> 6;
  *(char *)(DAT_0000e670 + 10) = (char)uVar3;
  uVar2 = (uint)uVar1;
  if (2 < uVar3) {
    uVar2 = DAT_0000e674;
  }
  uVar2 = uVar2 & DAT_0000e678;
  if (0x24 < uVar2) {
    uVar2 = 0x24;
  }
                    /* WARNING: Could not recover jumptable at 0x0000e5c0. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (**(code **)(DAT_0000e67c + uVar2 * 4))();
  return;
}



/* ======================================================================
 * 0000e5c2  hi_msg_release
 * ====================================================================== */

void hi_msg_release(uint param_1)

{
  int iVar1;
  uint *puVar2;
  uint uVar3;
  
  iVar1 = DAT_0000e684;
  uVar3 = DAT_0000e678 & *(ushort *)(param_1 + 2);
  if (uVar3 == DAT_0000e680) {
    if (0x17 < *(uint *)(DAT_0000e684 + 0x40)) {
      *DAT_0000e688 = *DAT_0000e688 & 0xfffffff7;
      evt_flags_set(DAT_0000e68c,0x80000);
    }
    *(int *)(iVar1 + 0x40) = *(int *)(iVar1 + 0x40) + -1;
    rx_buf_release(param_1 + 0x10,*(uint *)(param_1 + 0xc) & 0x40);
    return;
  }
  if ((uVar3 == DAT_0000e680 + 1) && (*(int *)(param_1 + 4) == 4)) {
    *(undefined1 *)(DAT_0000e690 + 0x13) = 0;
  }
  puVar2 = DAT_0000e694;
  uVar3 = *DAT_0000e694;
  if (4 < (int)(uVar3 - DAT_0000e694[1])) {
    fw_assert(s_hi_msg_c_0000e698,0x1a8,7);
  }
  puVar2[(uVar3 & 3) + 2] = param_1;
  *puVar2 = uVar3 + 1;
  return;
}



/* ======================================================================
 * 0000e63c  hif_alloc_msg_to_host
 * ====================================================================== */

/* hif_alloc_msg_to_host(len) -- allocates a firmware->host message buffer.
   
   Every indication this firmware can emit is built through here (except
   0x0801 STARTUP and 0x0804 RECEIVE, which use their own paths), so its
   caller list is the COMPLETE indication inventory:
   
     FUN_0000371C, FUN_000039B2   -> 0x0805 event
     FUN_00011212, FUN_00011260   -> 0x0806 scan complete
     FUN_000119D0, FUN_00011A0A,
     FUN_00011A38, FUN_00011A66   -> 0x080F join complete
     FUN_00012974                 -> 0x080D  (see below)
   
   Complete emitted set: 0x0801, 0x0804, 0x0805, 0x0806, 0x080D, 0x080F.
   
   Compared with mainline cw1200's wsm_handle_rx switch:
     * mainline ALSO handles 0x0808 ba_timeout, 0x0809 set_pm,
       0x080A channel_switch, 0x080B find_complete, 0x080C suspend_resume --
       this firmware never sends any of them, so those cases are dead code on
       XR819.  0x080B being dead is consistent with commands 0x0019/0x001A
       (start/stop find) being unsupported.
     * mainline does NOT handle 0x080D, so if it is ever emitted the driver
       logs "Unrecognised WSM ID 080d".
   
   *** THERE IS NO MEASUREMENT-COMPLETE INDICATION IN THIS BUILD. ***
   No function in the 802.11k measurement module calls this allocator or
   hif_send_msg_to_host.  Command 0x000E accepts the request, sets
   g_measure_ctl and signals event 0x2000, but no result can ever reach the
   host.  So the 11k measurement API is not usable as a source of survey data
   (channel load / noise) on this firmware, however complete the vendor
   header's MEASUREMENT_COMPLETE struct looks.
   
   0x080D (ind_080D_tx_trace, 0x00012974) is emitted only from FUN_0000B054 in
   the TX path, gated on a mode byte == 5, and forwards a payload with a
   64-bit TSF at +8 -- a TX trace/debug channel, not a measurement report. */

int hif_alloc_msg_to_host(uint param_1)

{
  int iVar1;
  uint uVar2;
  
  if (0x180 < param_1) {
    fw_assert(s_hi_msg_c_0000e698,0x1c2,8);
  }
  uVar2 = DAT_0000e694[1];
  if (0 < (int)(*DAT_0000e694 - uVar2)) {
    iVar1 = DAT_0000e694[(uVar2 & 3) + 2];
    DAT_0000e694[1] = uVar2 + 1;
    return iVar1;
  }
  return 0;
}



/* ======================================================================
 * 0000e6a4  fw_delay_loop
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void fw_delay_loop(int param_1)

{
  do {
  } while (-param_1 < 1);
  return;
}



/* ======================================================================
 * 0000e6b8  fw_read_timer
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

int fw_read_timer(void)

{
  return _DAT_0ac00004 + *(int *)(DAT_0000e6c8 + 0x14);
}



/* ======================================================================
 * 0000e6cc  hif_run_deferred_callbacks
 * ====================================================================== */

void hif_run_deferred_callbacks(void)

{
  int *piVar1;
  int *piVar2;
  
  piVar1 = DAT_0000eab8;
  while( true ) {
    irq_disable_save();
    piVar2 = (int *)piVar1[3];
    if (piVar2 == (int *)0x0) break;
    piVar1[3] = *piVar2;
    irq_restore();
    (*(code *)piVar2[9])(piVar2);
  }
  irq_restore();
  if (*piVar1 == 0) {
    evt_flags_clear(8);
  }
  return;
}



/* ======================================================================
 * 0000e6fc  hif_hw_program_xfer
 * ====================================================================== */

void hif_hw_program_xfer(int param_1,int param_2)

{
  ushort uVar1;
  uint *puVar2;
  uint uVar3;
  
  *(undefined1 *)(param_1 + 0x48) = 1;
  puVar2 = DAT_0000eabc;
  do {
  } while (-1 < (int)(*DAT_0000eabc << 0x12));
  DAT_0000eabc[1] = *(uint *)(param_1 + 0x48);
  puVar2[1] = *(uint *)(param_1 + 0x4c);
  puVar2[1] = *(uint *)(param_1 + 0x50);
  puVar2[1] = *(uint *)(param_1 + 0x54);
  do {
  } while ((int)(*puVar2 << 0x13) < 0);
  *puVar2 = 0x1240;
  do {
  } while (-1 < (int)(*puVar2 << 0x12));
  puVar2[1] = *(uint *)(param_1 + 0x28);
  puVar2[1] = *(uint *)(param_1 + 0x2c);
  puVar2[1] = *(uint *)(param_1 + 0x30);
  puVar2[1] = *(uint *)(param_1 + 0x34);
  do {
  } while ((int)(*puVar2 << 0x13) < 0);
  uVar3 = DAT_0000eac0;
  if (-1 < param_2 << 0x1d) {
    uVar3 = DAT_0000eac0 - 1;
  }
  *puVar2 = uVar3;
  uVar1 = *(ushort *)(param_1 + 0x28);
  do {
  } while (-1 < (int)(*puVar2 << 0x12));
  puVar2[1] = *(uint *)(param_1 + 0x38);
  puVar2[1] = *(uint *)(param_1 + 0x3c);
  puVar2[1] = *(uint *)(param_1 + 0x40);
  puVar2[1] = *(uint *)(param_1 + 0x44);
  do {
  } while ((int)(*puVar2 << 0x13) < 0);
  if (param_2 << 0x1d < 0) {
    uVar3 = DAT_0000eac0 + 8;
  }
  else {
    uVar3 = DAT_0000eac0 + 7;
  }
  *puVar2 = ((uVar1 >> 8) + 2 & 0xf) << 4 | uVar3;
  return;
}



/* ======================================================================
 * 0000e794  crypto_hw_program_ccmp
 * ====================================================================== */

undefined8 crypto_hw_program_ccmp(uint param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  uint *puVar1;
  uint uVar2;
  uint uVar3;
  uint *puVar4;
  uint uVar5;
  ushort *puVar6;
  ushort *puVar7;
  bool bVar8;
  uint local_24;
  int local_20;
  undefined4 local_1c;
  undefined4 local_18;
  
  puVar1 = DAT_0000eabc;
  local_18 = param_4;
  local_1c = param_3;
  local_20 = param_2;
  local_24 = param_1;
  do {
  } while (-1 < (int)(*DAT_0000eabc << 0x12));
  DAT_0000eabc[1] = *(uint *)(param_1 + 0x48);
  puVar1[1] = *(uint *)(param_1 + 0x4c);
  puVar1[1] = *(uint *)(param_1 + 0x50);
  puVar1[1] = *(uint *)(param_1 + 0x54);
  do {
  } while ((int)(*puVar1 << 0x13) < 0);
  *puVar1 = DAT_0000eac4;
  puVar6 = *(ushort **)(param_1 + 0x1c);
  do {
  } while (-1 < (int)(*puVar1 << 0x12));
  uVar3 = 0;
  do {
    puVar7 = puVar6 + -1;
    puVar6 = puVar6 + -2;
    puVar1[1] = (*puVar7 & 0xff) << 8 | (uint)(*puVar7 >> 8) | (uint)(*puVar6 >> 8) << 0x10 |
                (uint)*puVar6 << 0x18;
    uVar3 = uVar3 + 1;
  } while (uVar3 < 4);
  do {
  } while ((int)(*puVar1 << 0x13) < 0);
  *puVar1 = 0x1200;
  uVar3 = DAT_0000eac0;
  if (-1 < param_2 << 0x1d) {
    uVar3 = DAT_0000eac0 - 1;
  }
  bVar8 = (*(ushort *)(param_1 + 0x28) & 0x8c) != 0x88;
  uVar2 = 0;
  puVar4 = (uint *)(param_1 + 0x28);
  do {
    do {
      do {
      } while (-1 < (int)(*puVar1 << 0x12));
      uVar5 = 0;
      do {
        puVar1[1] = *puVar4;
        puVar4 = puVar4 + 1;
        uVar5 = uVar5 + 1;
      } while (uVar5 < 4);
      if (uVar2 == 1) {
        if (bVar8) {
          uVar5 = 8;
LAB_0000e85a:
          uVar3 = uVar3 | uVar5;
        }
      }
      else if (uVar2 == 2) {
        uVar5 = 0x28;
        goto LAB_0000e85a;
      }
      do {
      } while ((int)(*puVar1 << 0x13) < 0);
      *puVar1 = uVar3;
      uVar2 = uVar2 + 1;
      if (2 < uVar2) goto LAB_0000e86a;
    } while (uVar2 != 2);
    if (bVar8) {
LAB_0000e86a:
      return CONCAT44(local_20,local_24);
    }
    local_24 = *(uint *)(param_1 + 0x20) >> 8 | (*(uint *)(param_1 + 0x20) & 0xff) << 8;
    local_20 = 0;
    local_1c = 0;
    local_18 = 0;
    puVar4 = &local_24;
  } while( true );
}



/* ======================================================================
 * 0000e86c  crypto_hw_program_key
 * ====================================================================== */

void crypto_hw_program_key(int param_1,int param_2)

{
  uint *puVar1;
  uint uVar2;
  uint uVar3;
  
  puVar1 = DAT_0000eabc;
  do {
  } while ((int)(*DAT_0000eabc << 0x13) < 0);
  DAT_0000eabc[1] = *(uint *)(param_1 + 8);
  puVar1[1] = *(uint *)(param_1 + 0xc);
  puVar1[1] = *(uint *)(param_1 + 0x10);
  puVar1[1] = *(uint *)(param_1 + 0x14);
  *puVar1 = 0x1100;
  if (param_2 << 0x16 < 0) {
    crypto_hw_program_ccmp(param_1,param_2);
  }
  else if (param_2 << 0x15 < 0) {
    do {
    } while ((int)(*puVar1 << 0x13) < 0);
    *puVar1 = DAT_0000eac8;
    puVar1[1] = *(uint *)(param_1 + 0x28);
    puVar1[1] = *(uint *)(param_1 + 0x2c);
    puVar1[1] = *(uint *)(param_1 + 0x30);
    puVar1[1] = *(uint *)(param_1 + 0x34);
    do {
    } while ((int)(*puVar1 << 0x13) < 0);
    uVar2 = DAT_0000eac0 + 1;
    *puVar1 = uVar2;
    do {
    } while (-1 < (int)(*puVar1 << 0x12));
    puVar1[1] = *(uint *)(param_1 + 0x38);
    puVar1[1] = *(uint *)(param_1 + 0x3c);
    puVar1[1] = *(uint *)(param_1 + 0x40);
    puVar1[1] = *(uint *)(param_1 + 0x44);
    do {
    } while ((int)(*puVar1 << 0x13) < 0);
    *puVar1 = uVar2;
  }
  else {
    hif_hw_program_xfer(param_1,param_2);
  }
  uVar2 = *(uint *)(param_1 + 0x1c) & 0xf6ffffff;
  if (param_2 << 0x15 < 0) {
    uVar2 = uVar2 + 0xc;
  }
  puVar1[4] = uVar2;
  uVar2 = *(uint *)(param_1 + 0x20);
  if (param_2 << 0x15 < 0) {
    uVar2 = uVar2 - 0xc;
  }
  puVar1[6] = uVar2;
  puVar1[5] = *(uint *)(param_1 + 0x18) & 0xf6ffffff;
  if (param_2 << 0x15 < 0) {
    uVar2 = DAT_0000eacc | (0x10 - (*(uint *)(param_1 + 0x20) & 0xf) & 0x1f) * 0x10000 + 4;
  }
  else {
    uVar2 = DAT_0000eacc;
    if (param_2 << 0x1d < 0) {
      uVar2 = DAT_0000eacc + 1;
    }
    if (param_2 << 0x16 < 0) {
      uVar3 = 0x100000;
    }
    else {
      uVar3 = 0x80000;
    }
    uVar2 = uVar2 | uVar3;
  }
  do {
  } while ((int)(*puVar1 << 0x13) < 0);
  *puVar1 = uVar2;
  return;
}



/* ======================================================================
 * 0000e950  hif_defer_xfer
 * ====================================================================== */

void hif_defer_xfer(undefined4 *param_1)

{
  int iVar1;
  
  iVar1 = DAT_0000eab8;
  *(undefined4 *)(DAT_0000eab8 + 4) = *param_1;
  *param_1 = 0;
  if (*(int *)(iVar1 + 0xc) == 0) {
    *(undefined4 **)(iVar1 + 0xc) = param_1;
  }
  else {
    **(undefined4 **)(iVar1 + 0x10) = param_1;
  }
  *(undefined4 **)(iVar1 + 0x10) = param_1;
  evt_flags_set(DAT_0000ead0,0x10000000);
  return;
}



/* ======================================================================
 * 0000e978  hif_start_next_xfer
 * ====================================================================== */

void hif_start_next_xfer(void)

{
  undefined4 *puVar1;
  uint *puVar2;
  int iVar3;
  uint uVar4;
  
  puVar1 = DAT_0000eab8;
  while( true ) {
    iVar3 = puVar1[1];
    if (iVar3 == 0) {
      return;
    }
    if (*(int *)(iVar3 + 0x20) != 0) break;
    hif_defer_xfer(iVar3);
  }
  *puVar1 = 1;
  uVar4 = *(uint *)(DAT_0000ead4 + (uint)*(byte *)(iVar3 + 4) * 4);
  if (*(char *)(puVar1 + 5) != '\x02') {
    if ((int)(uVar4 << 0x16) < 0) {
      dbg_print_banner();
      goto LAB_0000e9be;
    }
    if (*(char *)(puVar1 + 5) == '\x01') goto LAB_0000e9be;
  }
  if ((uVar4 & 0x480) != 0) {
    dbg_print_help_text();
  }
LAB_0000e9be:
  puVar2 = DAT_0000ead8;
  if ((uVar4 & 0x680) != 0) {
    crypto_hw_program_key(iVar3,uVar4);
    return;
  }
  if ((int)(uVar4 * 0x40000000) < 0) {
    DAT_0000ead8[7] = *(uint *)(iVar3 + 8);
    puVar2[8] = *(uint *)(iVar3 + 0xc);
    puVar2[9] = *(uint *)(iVar3 + 0x10);
    puVar2[10] = *(uint *)(iVar3 + 0x14);
    puVar2[6] = (uint)*(ushort *)(iVar3 + 6);
  }
  puVar2[2] = *(uint *)(iVar3 + 0x1c) & 0xf6ffffff;
  puVar2[3] = *(uint *)(iVar3 + 0x18) & 0xf6ffffff;
  puVar2[5] = *(uint *)(iVar3 + 0x20);
  *puVar2 = uVar4 & 0x7f;
  return;
}



/* ======================================================================
 * 0000ea08  enc_xfer_complete
 * ====================================================================== */

void enc_xfer_complete(void)

{
  undefined4 *puVar1;
  int iVar2;
  
  puVar1 = DAT_0000eab8;
  *DAT_0000eab8 = 0;
  iVar2 = puVar1[1];
  if (iVar2 == 0) {
    fw_assert(s_enc_c_0000eae0,DAT_0000eadc,1);
  }
  *(undefined1 *)(iVar2 + 5) = 0;
  if (*(byte *)(iVar2 + 4) < 6) {
    if (-1 < *(int *)(DAT_0000ead8 + 4) << 0x1e) goto LAB_0000ea42;
  }
  else if ((*(byte *)(iVar2 + 4) == 10) || ((*DAT_0000eabc & 1) != 0)) goto LAB_0000ea42;
  *(undefined1 *)(iVar2 + 5) = 1;
LAB_0000ea42:
  hif_defer_xfer(iVar2);
  hif_start_next_xfer();
  return;
}



/* ======================================================================
 * 0000ea4e  hif_submit_or_queue
 * ====================================================================== */

void hif_submit_or_queue(undefined4 *param_1)

{
  int *piVar1;
  undefined4 uVar2;
  uint *puVar3;
  
  piVar1 = DAT_0000eab8;
  puVar3 = param_1 + -0x24;
  if (((param_1[8] == 0) && (DAT_0000eab8[1] == 0)) && (DAT_0000eab8[3] == 0)) {
    *puVar3 = *puVar3 | 0x10;
    (*(code *)param_1[9])(param_1);
    return;
  }
  *param_1 = 0;
  uVar2 = irq_disable_save();
  if (piVar1[1] == 0) {
    piVar1[1] = (int)param_1;
  }
  else {
    *(undefined4 **)piVar1[2] = param_1;
  }
  piVar1[2] = (int)param_1;
  if (*piVar1 == 0) {
    *puVar3 = *puVar3 | 0x10;
    hif_start_next_xfer();
  }
  irq_restore(uVar2);
  evt_flags_set((uint *)(DAT_0000ead0 + 4),8);
  return;
}



/* ======================================================================
 * 0000eb30  hif_queue_msg_to_host
 * ====================================================================== */

void hif_queue_msg_to_host(uint param_1)

{
  ushort uVar1;
  int iVar2;
  uint *puVar3;
  uint uVar4;
  int iVar5;
  uint *puVar6;
  
  iVar5 = DAT_0000ee44;
  puVar3 = DAT_0000ee40;
  iVar2 = DAT_0000ee30;
  puVar6 = (uint *)(DAT_0000ee30 + -0x10);
  uVar4 = *puVar6 - *(int *)(DAT_0000ee30 + -0xc);
  *DAT_0000ee40 = uVar4 & 0x1f;
  puVar3[1] = (uint)*(ushort *)(iVar5 + 4);
  if (0x1f < uVar4) {
    fw_assert(s_hif_c_0000ee38,0x2a8,5);
  }
  *(uint *)((*puVar6 & 0x1f) * 4 + iVar5 + 0x20) = param_1;
  uVar1 = *(ushort *)(DAT_0000ee44 + 0xa4);
  uVar4 = *puVar6;
  iVar5 = (*(uint *)(iVar2 + -8) & uVar4) * 8;
  *(uint *)(*(int *)(iVar2 + -4) + iVar5) = param_1 & 0xf6ffffff;
  *(uint *)(*(int *)(iVar2 + -4) + iVar5 + 4) = uVar1 + 1 & 0x1fff | 1;
  *puVar6 = uVar4 + 1;
  return;
}



/* ======================================================================
 * 0000eb96  hif_rx_process
 * ====================================================================== */

void hif_rx_process(void)

{
  ushort uVar1;
  bool bVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  uint *puVar8;
  ushort *puVar9;
  
  iVar3 = DAT_0000ee30;
  bVar2 = false;
  puVar8 = (uint *)(DAT_0000ee30 + -0x10);
  uVar7 = *(uint *)(DAT_0000ee30 + -0xc);
  while( true ) {
    if (*puVar8 == uVar7) {
      return;
    }
    uVar6 = *(uint *)(iVar3 + -8) & uVar7;
    uVar5 = *(uint *)(*(int *)(iVar3 + -4) + uVar6 * 8 + 4);
    if ((uVar5 & 1) != 0) break;
    if (bVar2) {
      evt_flags_set(DAT_0000ee28,0x2000000);
      return;
    }
    puVar9 = *(ushort **)(uVar6 * 4 + DAT_0000ee44 + 0x20);
    if ((uVar5 & DAT_0000ee48) < (uint)*puVar9) {
      fw_assert(s_hif_c_0000ee38,DAT_0000ee34 + 0xa0,6);
    }
    uVar1 = puVar9[1];
    if ((uint)(uVar1 >> 0xd) != (uVar7 & 7)) {
      fw_assert(s_hif_c_0000ee38,DAT_0000ee34 + 0xaa,0x32);
    }
    if ((DAT_0000ee4c & uVar1) != 4) {
      *(uint *)(DAT_0000ee50 + 0x10) = (uint)uVar1;
    }
    puVar9[1] = uVar1 & 0x1fff;
    iVar4 = DAT_0000ee44;
    uVar7 = uVar7 + 1;
    *(uint *)(iVar3 + -0xc) = uVar7;
    bVar2 = true;
    *(int *)(iVar4 + 0xa0) = *(int *)(iVar4 + 0xa0) + 1;
    trace_push_event(puVar9 + 1);
    wsm_dispatch_cmd(puVar9);
  }
  return;
}



/* ======================================================================
 * 0000ec30  hif_confirm_coalesce_hold
 * ====================================================================== */

/* hif_confirm_coalesce_hold() -- TX-CONFIRM INTERRUPT COALESCING.
   Returns 1 = hold (defer notifying the host), 0 = release.
   
   Called from hif_tx_confirm_drain (0x0000ED38), which runs the TX completion
   processing (tx_complete_tala_adapt) ONLY when this returns 0.  So this gates
   delivery of TX confirms to the host -- and TX confirms are what return the
   host's buffer credits (30 x 1632).
   
   Config block at 0x040011AC, in INITIALISED data, so these are the power-on
   defaults and no driver changes them:
       +0x08 enable        = 1
       +0x09 thr_pending   = 10     (TX-done ring producer-consumer)
       +0x0A thr_ringdepth = 4
       +0x0B thr_count     = 2
       +0x0C delay         = 0x1F40 = 8000 fw_read_timer ticks
   
   Logic: if enable == 0 or delay == 0, never hold.  Otherwise, while ALL of
   the three depths are still <= their thresholds, hold -- starting a timestamp
   on the first hold and releasing once `delay` ticks have elapsed.  As soon as
   any depth exceeds its threshold, release immediately.
   
   So it batches interrupts when the link is idle and self-disables under load,
   which is sane.  Worth measuring anyway: it sits directly in the credit-return
   path, and the delay is fixed in firmware with no MIB known to write it.
   Pokeable at runtime via cmd 0x0001 or MIB 0x0009.
   
   Related: hif_queue_msg_to_host (0x0000EB30) pushes into a 32-entry ring and
   asserts hif.c:680 code 5 if the depth exceeds 31. */

undefined4 hif_confirm_coalesce_hold(void)

{
  int iVar1;
  int iVar2;
  undefined4 uVar3;
  int iVar4;
  int iVar5;
  
  iVar4 = DAT_0000ee54;
  if (*(int *)(DAT_0000ee54 + 0x10) == *(int *)(DAT_0000ee54 + 0xc)) {
    return 1;
  }
  uVar3 = irq_fiq_disable_save();
  iVar2 = DAT_0000ee44;
  iVar1 = DAT_0000ee40;
  if ((*(char *)(DAT_0000ee40 + 8) == '\0') || (*(int *)(DAT_0000ee40 + 0xc) == 0)) {
    *(undefined4 *)(DAT_0000ee44 + 8) = 0;
  }
  else {
    iVar5 = DAT_0000ee44 + 0xc;
    if (((*(int *)(iVar4 + 0x10) - *(int *)(iVar4 + 0xc) <= (int)(uint)*(byte *)(DAT_0000ee40 + 9))
        && ((ushort)*(byte *)(DAT_0000ee40 + 0xb) <= *(ushort *)(DAT_0000ee44 + 4))) &&
       ((int)(uint)*(byte *)(DAT_0000ee40 + 10) <=
        *(int *)(DAT_0000ee30 + -0x10) - *(int *)(DAT_0000ee30 + -0xc))) {
      if (*(int *)(DAT_0000ee44 + 8) == 0) {
        iVar4 = fw_read_timer();
        if (iVar4 == 0) {
          iVar4 = 1;
        }
        *(int *)(iVar2 + 8) = iVar4;
        if (*(char *)(iVar1 + 0xb) == '\0') {
          timer_start(iVar5,*(undefined4 *)(iVar1 + 0xc));
        }
LAB_0000ecd2:
        irq_fiq_restore(uVar3);
        return 1;
      }
      iVar4 = fw_read_timer();
      if (-1 < (*(int *)(iVar2 + 8) + *(int *)(iVar1 + 0xc)) - iVar4) goto LAB_0000ecd2;
    }
    *(undefined4 *)(iVar2 + 8) = 0;
    if (*(char *)(iVar1 + 0xb) == '\0') {
      timer_cancel(iVar5);
    }
  }
  irq_fiq_restore(uVar3);
  return 0;
}



/* ======================================================================
 * 0000ecda  hif_push_next_with_seq
 * ====================================================================== */

void hif_push_next_with_seq(void)

{
  ushort uVar1;
  uint *puVar2;
  int iVar3;
  uint uVar4;
  ushort *puVar5;
  
  puVar2 = DAT_0000ee30;
  uVar4 = *DAT_0000ee30;
  if (DAT_0000ee30[-6] != uVar4) {
    puVar5 = *(ushort **)((uVar4 & 0x3f) * 4 + DAT_0000ee44 + 0xa8);
    uVar1 = puVar5[1];
    trace_push_event_filtered((uint)uVar1);
    puVar5[1] = (ushort)(uVar4 << 0xd) | uVar1 & 0x9fff;
    iVar3 = (uVar4 & 3) * 8;
    *(uint *)(puVar2[3] + iVar3) = (uint)puVar5 & 0xf6ffffff;
    *(uint *)(puVar2[3] + iVar3 + 4) = uVar1 & 0x6000 | *puVar5 + 1 & 0x1fff | 1;
    *puVar2 = uVar4 + 1;
  }
  return;
}



/* ======================================================================
 * 0000ed38  hif_tx_confirm_drain
 * ====================================================================== */

void hif_tx_confirm_drain(void)

{
  ushort uVar1;
  uint *puVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  
  iVar4 = DAT_0000ee44;
  puVar2 = DAT_0000ee30;
  uVar5 = DAT_0000ee30[1];
  while (*puVar2 != uVar5) {
    if ((*(uint *)(puVar2[3] + (puVar2[2] & uVar5) * 8 + 4) & 1) != 0) break;
    uVar5 = uVar5 + 1;
    puVar2[1] = uVar5;
    hif_push_next_with_seq();
    iVar3 = *(int *)((DAT_0000ee30[-5] & 0x3f) * 4 + iVar4 + 0xa8);
    DAT_0000ee30[-5] = DAT_0000ee30[-5] + 1;
    *(short *)(iVar4 + 4) = *(short *)(iVar4 + 4) + -1;
    uVar1 = *(ushort *)(iVar3 + 2);
    if ((int)((uint)uVar1 << 0x14) < 0) {
      *(ushort *)(iVar3 + 2) = uVar1 & 0x1fff;
      hi_msg_release();
    }
    if (*(short *)(iVar4 + 4) == 0) {
      evt_flags_clear(2);
    }
  }
  iVar4 = hif_confirm_coalesce_hold();
  if (iVar4 == 0) {
    tx_complete_tala_adapt();
  }
  return;
}



/* ======================================================================
 * 0000eda4  hif_send_msg_to_host
 * ====================================================================== */

void hif_send_msg_to_host(int param_1)

{
  int *piVar1;
  int iVar2;
  short sVar3;
  uint uVar4;
  
  if (param_1 == 0) {
    fw_assert(s_hif_c_0000ee38,400,4);
  }
  if (-1 < (int)((uint)*(ushort *)(param_1 + 2) << 0x14)) {
    *(ushort *)(param_1 + 2) = *(ushort *)(param_1 + 2) | 0x400;
  }
  iVar2 = DAT_0000ee44;
  sVar3 = *(short *)(DAT_0000ee44 + 4) + 1;
  *(short *)(DAT_0000ee44 + 4) = sVar3;
  if (sVar3 == 1) {
    evt_flags_set((uint *)(DAT_0000ee28 + 4),2);
  }
  piVar1 = DAT_0000ee30;
  uVar4 = DAT_0000ee30[-6];
  if (0x3f < uVar4 - DAT_0000ee30[-5]) {
    fw_assert(s_hif_c_0000ee38,0x1b5,0x30);
  }
  if ((int)((uint)*(ushort *)(param_1 + 2) << 0x15) < 0) {
    hif_queue_msg_to_host(param_1);
  }
  *(int *)((uVar4 & 0x3f) * 4 + iVar2 + 0xa8) = param_1;
  piVar1[-6] = uVar4 + 1;
  if (*DAT_0000ee30 - DAT_0000ee30[1] < 4) {
    hif_push_next_with_seq();
  }
  return;
}



/* ======================================================================
 * 0000ee58  hif_irq_demux
 * ====================================================================== */

/* hif_irq_demux(arg) -- SECOND-LEVEL HARDWARE INTERRUPT DEMUX.
   Reached from the ARM IRQ/FIQ vector trampoline at 0x0000003C (blx 0x0000EE58).
   
     pending = *(u32 *)(base + 0x20);
     *(u32 *)(base + 4) = pending;              /* ack */
     while (pending) {
         bit = fw_clz(pending);
         handler_tbl[bit]();                    /* table at DAT_0000EE8C, 32 slots */
         pending &= ~(mask >> bit);
     }
   
   Note this is a SEPARATE dispatch table from the cooperative scheduler's
   g_sched_handler_tbl (0x040021B4) used by sched_main_loop (0x0000F198).
   Two levels:
     * hardware IRQ  -> hif_irq_demux -> per-source ISR (this table)
     * an ISR sets an event flag -> sched_main_loop -> task (the other table)
   
   `mac_irq_handler` (0x00009EB4) is the MAC event ISR reached from vector 0x24,
   i.e. the first level; tasks like rx_handler and tx_complete_tala_adapt run at
   the second. */

void hif_irq_demux(undefined4 param_1)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  
  *(undefined4 *)(DAT_0000ee84 + 0xc) = param_1;
  iVar1 = DAT_0000ee88;
  uVar4 = *(uint *)(DAT_0000ee88 + 0x20);
  *(uint *)(DAT_0000ee88 + 4) = uVar4;
  iVar2 = DAT_0000ee8c;
  for (; uVar4 != 0; uVar4 = uVar4 & ~((uint)(iVar1 << 0xc) >> (uVar3 & 0xff))) {
    uVar3 = fw_clz(uVar4);
    (**(code **)(iVar2 + uVar3 * 4))();
  }
  return;
}



/* ======================================================================
 * 0000ee90  task_hif_ee90
 * ====================================================================== */

void task_hif_ee90(void)

{
  int *piVar1;
  int *piVar2;
  
  piVar1 = DAT_0000efc8;
  while (piVar2 = (int *)piVar1[3], piVar2 != (int *)0x0) {
    irq_disable_save();
    piVar1[3] = *piVar2;
    irq_restore();
    (*(code *)piVar2[5])(piVar2);
  }
  if (*piVar1 == 0) {
    evt_flags_clear(4);
  }
  return;
}



/* ======================================================================
 * 0000eebc  mic_finish_entry
 * ====================================================================== */

void mic_finish_entry(undefined4 *param_1)

{
  int iVar1;
  
  iVar1 = DAT_0000efc8;
  *(undefined4 *)(DAT_0000efc8 + 4) = *param_1;
  *param_1 = 0;
  if (*(int *)(iVar1 + 0xc) == 0) {
    *(undefined4 **)(iVar1 + 0xc) = param_1;
  }
  else {
    **(undefined4 **)(iVar1 + 0x10) = param_1;
  }
  *(undefined4 **)(iVar1 + 0x10) = param_1;
  evt_flags_set(DAT_0000efcc,0x20000000);
  return;
}



/* ======================================================================
 * 0000eee4  mic_start_next
 * ====================================================================== */

void mic_start_next(void)

{
  undefined4 *puVar1;
  int iVar2;
  
  puVar1 = DAT_0000efc8;
  while( true ) {
    iVar2 = puVar1[1];
    if (iVar2 == 0) {
      return;
    }
    if (*(int *)(iVar2 + 8) != 0) break;
    mic_finish_entry();
  }
  *puVar1 = 1;
  puVar1 = DAT_0000efd0;
  *DAT_0000efd0 = *(undefined4 *)(iVar2 + 0xc);
  puVar1[1] = *(undefined4 *)(iVar2 + 0x10);
  puVar1[2] = *(undefined4 *)(iVar2 + 8);
  puVar1[3] = *(uint *)(iVar2 + 4) & 0xf6ffffff;
  puVar1[4] = 0;
  return;
}



/* ======================================================================
 * 0000ef1c  mic_complete
 * ====================================================================== */

void mic_complete(void)

{
  undefined4 *puVar1;
  int iVar2;
  int iVar3;
  
  iVar2 = DAT_0000efd0;
  if (*(int *)(DAT_0000efd0 + 0x10) == 0) {
    fw_assert(s_mic_c_0000efd4,0x127,0x17);
  }
  puVar1 = DAT_0000efc8;
  *DAT_0000efc8 = 0;
  iVar3 = puVar1[1];
  if (iVar3 == 0) {
    fw_assert(s_mic_c_0000efd4,0x12d,0x18);
  }
  *(undefined4 *)(iVar3 + 0xc) = *(undefined4 *)(iVar2 + 0x14);
  *(undefined4 *)(iVar3 + 0x10) = *(undefined4 *)(iVar2 + 0x18);
  mic_finish_entry(iVar3);
  mic_start_next();
  return;
}



/* ======================================================================
 * 0000ef5e  mic_submit_or_queue
 * ====================================================================== */

void mic_submit_or_queue(undefined4 *param_1)

{
  int *piVar1;
  undefined4 uVar2;
  uint *puVar3;
  
  piVar1 = DAT_0000efc8;
  puVar3 = param_1 + -0x14;
  if (((param_1[2] == 0) && (DAT_0000efc8[1] == 0)) && (DAT_0000efc8[3] == 0)) {
    *puVar3 = *puVar3 | 4;
    (*(code *)param_1[5])(param_1);
    return;
  }
  *param_1 = 0;
  uVar2 = irq_disable_save();
  if (piVar1[1] == 0) {
    piVar1[1] = (int)param_1;
  }
  else {
    *(undefined4 **)piVar1[2] = param_1;
  }
  piVar1[2] = (int)param_1;
  if (*piVar1 == 0) {
    *puVar3 = *puVar3 | 4;
    mic_start_next();
  }
  irq_restore(uVar2);
  evt_flags_set((uint *)(DAT_0000efcc + 4),4);
  return;
}



/* ======================================================================
 * 0000efdc  irq_disable_save
 * ====================================================================== */

longlong irq_disable_save(void)

{
  char in_NG;
  char in_ZR;
  char in_CY;
  char in_OV;
  byte in_Q;
  
  return (ulonglong)
         ((uint)(byte)(in_NG << 4 | in_ZR << 3 | in_CY << 2 | in_OV << 1 | in_Q) << 0x1b | 0x80) <<
         0x20;
}



/* ======================================================================
 * 0000eff0  irq_restore
 * ====================================================================== */

void irq_restore(void)

{
  return;
}



/* ======================================================================
 * 0000f004  irq_fiq_disable_save
 * ====================================================================== */

longlong irq_fiq_disable_save(void)

{
  char in_NG;
  char in_ZR;
  char in_CY;
  char in_OV;
  byte in_Q;
  
  return (ulonglong)
         ((uint)(byte)(in_NG << 4 | in_ZR << 3 | in_CY << 2 | in_OV << 1 | in_Q) << 0x1b | 0xc0) <<
         0x20;
}



/* ======================================================================
 * 0000f018  irq_fiq_restore
 * ====================================================================== */

void irq_fiq_restore(void)

{
  return;
}



/* ======================================================================
 * 0000f02c  fw_clz
 * ====================================================================== */

int fw_clz(undefined4 param_1)

{
  return LZCOUNT(param_1);
}



/* ======================================================================
 * 0000f034  evt_flags_set
 * ====================================================================== */

void evt_flags_set(uint *flags,uint mask)

{
  *flags = *flags | mask;
  return;
}



/* ======================================================================
 * 0000f05c  cpu_wait_for_interrupt
 * ====================================================================== */

void cpu_wait_for_interrupt(undefined4 param_1)

{
  coproc_moveto_Wait_for_interrupt(param_1);
  return;
}



/* ======================================================================
 * 0000f064  fw_bit_length
 * ====================================================================== */

int fw_bit_length(int param_1)

{
  if (param_1 < 0) {
    param_1 = 0;
  }
  return 0x20 - LZCOUNT(param_1);
}



/* ======================================================================
 * 0000f0fc  fw_memcpy
 * ====================================================================== */

void fw_memcpy(void *dst,void *src,int n)

{
  undefined1 uVar1;
  bool bVar2;
  
  while (bVar2 = n != 0, n = n + -1, bVar2) {
    uVar1 = *(undefined1 *)src;
    src = (void *)((int)src + 1);
    *(undefined1 *)dst = uVar1;
    dst = (void *)((int)dst + 1);
  }
  return;
}



/* ======================================================================
 * 0000f106  fw_memcpy
 * ====================================================================== */

void fw_memcpy(void *dst,void *src,int n)

{
  undefined1 uVar1;
  bool bVar2;
  
  while (bVar2 = n != 0, n = n + -1, bVar2) {
    uVar1 = *(undefined1 *)src;
    src = (void *)((int)src + 1);
    *(undefined1 *)dst = uVar1;
    dst = (void *)((int)dst + 1);
  }
  return;
}



/* ======================================================================
 * 0000f10c  fw_memmove_rev
 * ====================================================================== */

void fw_memmove_rev(int param_1,int param_2,int param_3)

{
  undefined1 *puVar1;
  undefined1 *puVar2;
  bool bVar3;
  
  puVar2 = (undefined1 *)(param_2 + param_3);
  puVar1 = (undefined1 *)(param_1 + param_3);
  while( true ) {
    puVar2 = puVar2 + -1;
    puVar1 = puVar1 + -1;
    bVar3 = param_3 == 0;
    param_3 = param_3 + -1;
    if (bVar3) break;
    *puVar1 = *puVar2;
  }
  return;
}



/* ======================================================================
 * 0000f122  fw_memcpy_words
 * ====================================================================== */

void fw_memcpy_words(undefined4 *param_1,undefined4 *param_2,int param_3)

{
  undefined4 uVar1;
  bool bVar2;
  
  while (bVar2 = param_3 != 0, param_3 = param_3 + -1, bVar2) {
    uVar1 = *param_2;
    param_2 = param_2 + 1;
    *param_1 = uVar1;
    param_1 = param_1 + 1;
  }
  return;
}



/* ======================================================================
 * 0000f128  fw_memcpy_words
 * ====================================================================== */

void fw_memcpy_words(undefined4 *param_1,undefined4 *param_2,int param_3)

{
  undefined4 uVar1;
  bool bVar2;
  
  while (bVar2 = param_3 != 0, param_3 = param_3 + -1, bVar2) {
    uVar1 = *param_2;
    param_2 = param_2 + 1;
    *param_1 = uVar1;
    param_1 = param_1 + 1;
  }
  return;
}



/* ======================================================================
 * 0000f12e  fw_memzero
 * ====================================================================== */

void fw_memzero(void *dst,int n)

{
  bool bVar1;
  
  while (bVar1 = n != 0, n = n + -1, bVar1) {
    *(undefined1 *)dst = 0;
    dst = (void *)((int)dst + 1);
  }
  return;
}



/* ======================================================================
 * 0000f13c  fw_mem_equal
 * ====================================================================== */

undefined4 fw_mem_equal(char *param_1,int param_2,char *param_3,int param_4)

{
  char cVar1;
  char cVar2;
  bool bVar3;
  
  if (param_2 == param_4) {
    do {
      bVar3 = param_2 == 0;
      param_2 = param_2 + -1;
      if (bVar3) {
        return 1;
      }
      cVar1 = *param_1;
      cVar2 = *param_3;
      param_3 = param_3 + 1;
      param_1 = param_1 + 1;
    } while (cVar1 == cVar2);
  }
  return 0;
}



/* ======================================================================
 * 0000f15c  fw_streq
 * ====================================================================== */

undefined4 fw_streq(char *param_1,char *param_2)

{
  undefined4 uVar1;
  
  for (; *param_1 != '\0'; param_1 = param_1 + 1) {
    if (*param_1 != *param_2) goto LAB_0000f176;
    param_2 = param_2 + 1;
  }
  uVar1 = 1;
  if (*param_2 != '\0') {
LAB_0000f176:
    uVar1 = 0;
  }
  return uVar1;
}



/* ======================================================================
 * 0000f168  fw_streq
 * ====================================================================== */

undefined4 fw_streq(char *param_1,char *param_2)

{
  undefined4 uVar1;
  
  for (; *param_1 != '\0'; param_1 = param_1 + 1) {
    if (*param_1 != *param_2) goto LAB_0000f176;
    param_2 = param_2 + 1;
  }
  uVar1 = 1;
  if (*param_2 != '\0') {
LAB_0000f176:
    uVar1 = 0;
  }
  return uVar1;
}



/* ======================================================================
 * 0000f17a  fw_rand24_lfsr
 * ====================================================================== */

uint fw_rand24_lfsr(void)

{
  uint uVar1;
  uint uVar2;
  
  uVar1 = *(uint *)(DAT_0000f194 + 4);
  uVar2 = uVar1 >> 4 ^ uVar1;
  *(uint *)(DAT_0000f194 + 4) = uVar1 << 0x1b | uVar2 & 0x7ffffff;
  return uVar2 & 0xffffff;
}



/* ======================================================================
 * 0000f198  sched_main_loop
 * ====================================================================== */

/* sched_main_loop() -- THE FIRMWARE'S MAIN LOOP (scheduler.c).  Never returns.
   
     for (;;) {
         cpsr = irq_fiq_disable_save();
         while (*g_sched_event_flags == 0) {      // 0x04001FD4 + 4
             cpu_wait_for_interrupt();            // WFI
             irq_fiq_restore(cpsr);
             cpsr = irq_fiq_disable_save();
         }
         bit = fw_clz(*flags);                    // highest set bit wins
         *flags &= ~(0x80000000u >> bit);
         irq_fiq_restore(cpsr);
         trace(bit);
         fn = g_sched_handler_tbl[bit];           // 0x040021B4, 32 entries
         if (fn == Reset) fw_assert(scheduler.c, 135, 0x23);
         fn();
     }
   
   So the whole firmware is an **event-flag-driven cooperative scheduler**:
   every `evt_flags_set(g_sched_event_obj + 4, mask)` call wakes a task, and
   priority is strictly "highest bit first" via clz.  Nothing preempts a task
   except interrupts.
   
   The handler table is in bss, populated at runtime by sched_register_task
   (0x00016698): `g_sched_handler_tbl[fw_clz(mask)] = fn`.  Recovering its 21
   call sites gives the complete top-level task list:
   
     hif_rx_process (0x0000EB96)          host -> fw message dispatch
     hif_tx_confirm_drain (0x0000ED38)    TX completion + TALA
     hif_run_deferred_callbacks (0x0000E6CC)
     task_hif_ee90 (0x0000EE90)
     rx_handler (0x00008E2C)              RX frame processing
     tx_complete_tala_adapt (0x0000D254)
     bab_event_dispatch (0x00005B2A)      block-ack events
     task_timer_expiry (0x0000F204)
     task_measure_complete (0x00012CC2)   802.11k
     task_wsmlmac_11df0, task_11ecc, task_13B58, task_143A6,
     task_df4, task_2AB0, task_2DA0, task_22BC, task_7558,
     task_B88E, task_B9AE, task_15B70
   
   That the six already-named entries are exactly subsystem entry points is
   independent confirmation of those names.
   
   Mask -> bit assignments are only partly recovered (the mask argument is not
   reliably reconstructible from the call sites).  One is certain from the
   matching clear: mask 0x8 wakes hif_run_deferred_callbacks, which ends with
   evt_flags_clear(8). */

void sched_main_loop(void)

{
  uint *puVar1;
  undefined4 uVar2;
  uint uVar3;
  code *pcVar4;
  
  puVar1 = DAT_0000f1f0;
  do {
    while( true ) {
      uVar2 = irq_fiq_disable_save();
      if (*puVar1 != 0) break;
      cpu_wait_for_interrupt();
      irq_fiq_restore(uVar2);
    }
    uVar3 = fw_clz(*puVar1);
    *puVar1 = *puVar1 & ~(0x80000000U >> (uVar3 & 0xff));
    irq_fiq_restore(uVar2);
    trace_push_byte(uVar3 & 0xff);
    pcVar4 = *(code **)(DAT_0000f1f4 + uVar3 * 4);
    if (pcVar4 == Reset) {
      fw_assert(s_scheduler_c_0000f1f8,0x87,0x23);
    }
    (*pcVar4)();
  } while( true );
}



/* ======================================================================
 * 0000f26c  timer_cancel
 * ====================================================================== */

undefined4 timer_cancel(int *param_1)

{
  int iVar1;
  int *piVar2;
  
  piVar2 = (int *)param_1[1];
  if (piVar2 == (int *)0x0) {
    return 0;
  }
  if ((*(int *)(DAT_0000f314 + 4) << 0x1b < 0) && ((int *)*DAT_0000f310 == param_1)) {
    evt_flags_clear(0x10);
  }
  irq_fiq_disable_save();
  param_1[1] = 0;
  iVar1 = *param_1;
  *piVar2 = iVar1;
  if (iVar1 != 0) {
    *(int **)(iVar1 + 4) = piVar2;
  }
  irq_fiq_restore();
  return 1;
}



/* ======================================================================
 * 0000f2aa  timer_start
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void timer_start(undefined4 *param_1,int param_2)

{
  int *piVar1;
  int *piVar2;
  int iVar3;
  
  if (param_1[1] != 0) {
    timer_cancel(param_1);
  }
  if (param_2 < 0) {
    param_2 = 0;
  }
  iVar3 = _DAT_0ac00004 + *(int *)(DAT_0000f318 + 0x14) + param_2;
  irq_fiq_disable_save();
  piVar1 = DAT_0000f310;
  do {
    piVar2 = piVar1;
    piVar1 = (int *)*piVar2;
    if (piVar1 == (int *)0x0) goto LAB_0000f2e6;
  } while (piVar1[2] - iVar3 < 1);
  piVar1[1] = (int)param_1;
LAB_0000f2e6:
  param_1[2] = iVar3;
  *piVar2 = (int)param_1;
  *param_1 = piVar1;
  param_1[1] = piVar2;
  irq_fiq_restore();
  if (((undefined4 *)*DAT_0000f310 == param_1) && (DAT_0000f310[5] == 0)) {
    _DAT_0ac0001c = 0xc1;
    _DAT_0ac00014 = param_2;
  }
  return;
}



/* ======================================================================
 * 0000f31c  evt_flags_clear
 * ====================================================================== */

void evt_flags_clear(uint param_1)

{
  uint *puVar1;
  
  irq_fiq_disable_save();
  puVar1 = DAT_0000f340;
  DAT_0000f340[1] = DAT_0000f340[1] & ~param_1;
  if (puVar1[1] == 0) {
    *puVar1 = *puVar1 | 4;
  }
  irq_fiq_restore();
  return;
}



/* ======================================================================
 * 0000f344  fw_udelay_count
 * ====================================================================== */

void fw_udelay_count(int param_1)

{
  uint uVar1;
  
  for (uVar1 = 0; uVar1 <= (uint)(param_1 * 0x22) && param_1 * 0x22 - uVar1 != 0; uVar1 = uVar1 + 1)
  {
  }
  return;
}



/* ======================================================================
 * 0000f354  bab_find_session
 * ====================================================================== */

uint bab_find_session(uint param_1,int param_2)

{
  uint uVar1;
  int iVar2;
  
  uVar1 = 0;
  while ((((iVar2 = uVar1 * 0x28 + DAT_0000f3a8, *(int *)(iVar2 + 0x3a0) == 0 ||
           (*(short *)(param_2 + 10) != *(short *)(iVar2 + 0x3a4))) ||
          (*(short *)(param_2 + 0xc) != *(short *)(iVar2 + 0x3a6))) ||
         (((*(short *)(param_2 + 0xe) != *(short *)(iVar2 + 0x3a8) ||
           ((ushort)*(byte *)(iVar2 + 0x3aa) != (*(ushort *)(param_2 + 0x18) & 0xf))) ||
          (*(byte *)(iVar2 + 0x3ab) != param_1))))) {
    uVar1 = uVar1 + 1;
    if (3 < uVar1) {
      return uVar1;
    }
  }
  return uVar1;
}



/* ======================================================================
 * 0000f3da  link_state_init_all
 * ====================================================================== */

void link_state_init_all(void)

{
  undefined4 *puVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  puVar1 = DAT_0000f550;
  *DAT_0000f550 = 0;
  puVar1[8] = 0;
  puVar1 = DAT_0000f550;
  DAT_0000f550[0x10] = 0;
  puVar1[0x18] = 0;
  iVar4 = DAT_0000f558;
  *(undefined4 *)(DAT_0000f554 + 0x14) = 0;
  uVar2 = 0;
  do {
    iVar3 = uVar2 * 0x44;
    uVar2 = uVar2 + 1 & 0xff;
    *(undefined4 *)(iVar3 + iVar4 + 0x718) = 0;
    iVar3 = DAT_0000f54c;
  } while (uVar2 < 4);
  uVar2 = 0;
  do {
    iVar4 = uVar2 * 0x38 + iVar3;
    uVar2 = uVar2 + 1 & 0xff;
    *(undefined1 *)(iVar4 + 0x658) = 0;
    *(undefined2 *)(iVar4 + 0x65c) = 0x10;
    iVar4 = DAT_0000f55c;
  } while (uVar2 < 8);
  *(undefined1 *)(DAT_0000f55c + 0xc) = 0;
  *(undefined2 *)(iVar4 + 8) = 0;
  *(undefined2 *)(iVar4 + 10) = 0;
  return;
}



/* ======================================================================
 * 0000f436  link_remove_by_vif
 * ====================================================================== */

void link_remove_by_vif(uint param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  iVar1 = DAT_0000f54c;
  *(undefined4 *)(DAT_0000f554 + 0x14) = 0;
  for (uVar2 = 0; uVar2 < *(byte *)(iVar1 + 0x64c); uVar2 = uVar2 + 1 & 0xff) {
    iVar3 = uVar2 * 0x38 + iVar1;
    if ((*(byte *)(iVar3 + 0x677) == param_1) && (*(char *)(iVar3 + 0x658) != '\0')) {
      *(undefined1 *)(iVar3 + 0x658) = 0;
      *(char *)(iVar1 + 0x64c) = *(char *)(iVar1 + 0x64c) + -1;
    }
  }
  *(undefined2 *)(param_1 * 2 + iVar1 + 0x648) = 0;
  return;
}



/* ======================================================================
 * 0000f482  clear_field_718
 * ====================================================================== */

void clear_field_718(int param_1)

{
  *(undefined4 *)(param_1 * 0x44 + DAT_0000f558 + 0x718) = 0;
  return;
}



/* ======================================================================
 * 0000f496  clear_fields_718_and_20
 * ====================================================================== */

void clear_fields_718_and_20(int param_1)

{
  *(undefined4 *)(param_1 * 0x44 + DAT_0000f558 + 0x718) = 0;
  *(undefined4 *)(param_1 * 0x20 + DAT_0000f550) = 0;
  return;
}



/* ======================================================================
 * 0000f4c2  link_tbl_add_by_mac
 * ====================================================================== */

byte link_tbl_add_by_mac(char param_1,char param_2,undefined2 *param_3)

{
  byte bVar1;
  int iVar2;
  char *pcVar3;
  byte bVar4;
  
  iVar2 = DAT_0000f55c;
  bVar4 = 0;
  for (pcVar3 = (char *)(DAT_0000f55c + 0x18);
      (bVar4 < *(byte *)(DAT_0000f55c + 0xc) && (*pcVar3 != '\0')); pcVar3 = pcVar3 + 0x38) {
    bVar4 = bVar4 + 1;
  }
  if (bVar4 < 8) {
    *pcVar3 = '\x01';
    pcVar3[0x1f] = param_1;
    pcVar3[0x1e] = param_2;
    *(undefined2 *)(pcVar3 + 0x18) = *param_3;
    *(undefined2 *)(pcVar3 + 0x1a) = param_3[1];
    *(undefined2 *)(pcVar3 + 0x1c) = param_3[2];
    bVar1 = *(byte *)(iVar2 + 0xc);
    if (bVar1 == bVar4) {
      *(byte *)(iVar2 + 0xc) = bVar1 + 1;
    }
  }
  return bVar4;
}



/* ======================================================================
 * 0000f508  link_states_reset_to_1
 * ====================================================================== */

void link_states_reset_to_1(void)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  iVar1 = DAT_0000f54c;
  for (uVar2 = 0; uVar2 < *(byte *)(iVar1 + 0x64c); uVar2 = uVar2 + 1 & 0xff) {
    iVar3 = uVar2 * 0x38 + iVar1;
    if (*(byte *)(iVar3 + 0x658) < 6) {
      *(ushort *)(iVar1 + 0x818) = *(ushort *)(iVar1 + 0x818) & ~(ushort)(1 << uVar2);
      *(undefined1 *)(iVar3 + 0x658) = 1;
    }
  }
  return;
}



/* ======================================================================
 * 0000f564  mac_hw_init_pipes
 * ====================================================================== */

void mac_hw_init_pipes(void)

{
  undefined1 uVar1;
  undefined4 *puVar2;
  int iVar3;
  uint uVar4;
  int *piVar5;
  byte bVar6;
  uint uVar7;
  uint uVar8;
  int iVar9;
  int iVar10;
  int iVar11;
  int iVar12;
  
  puVar2 = DAT_0000f6f8;
  uVar8 = 0;
  DAT_0000f6f8[3] = 0;
  DAT_0000f6f8[-8] = 2;
  *puVar2 = 0x100;
  puVar2[2] = 0xff;
  iVar3 = DAT_0000f700;
  *(uint *)(DAT_0000f700 + 0xc) = DAT_0000f6fc & 0xf6ffffff;
  *(undefined4 *)(iVar3 + 0x10) = 0x54;
  *(undefined4 *)(iVar3 + 0x14) = 1;
  iVar11 = DAT_0000f700;
  iVar10 = DAT_0000f700 + 0x80;
  *(uint *)(DAT_0000f700 + 0x8c) = DAT_0000f704 & 0xf6ffffff;
  *(undefined4 *)(iVar11 + 0x90) = 0x54;
  *(undefined4 *)(iVar11 + 0x94) = 1;
  iVar11 = DAT_0000f70c;
  *(uint *)(DAT_0000f70c + 0xc) = DAT_0000f708 & 0xf6ffffff;
  *(undefined4 *)(iVar11 + 0x10) = 0x54;
  *(undefined4 *)(iVar11 + 0x14) = 1;
  iVar9 = DAT_0000f70c;
  iVar12 = DAT_0000f70c + 0x80;
  *(uint *)(DAT_0000f70c + 0x8c) = DAT_0000f710 & 0xf6ffffff;
  *(undefined4 *)(iVar9 + 0x90) = 0x54;
  *(undefined4 *)(iVar9 + 0x94) = 1;
  iVar9 = DAT_0000f714;
  uVar7 = 0;
  *(int *)(DAT_0000f714 + 0x28) = iVar3;
  *(int *)(iVar9 + 0x94) = iVar10;
  *(int *)(iVar9 + 0x100) = iVar11;
  *(int *)(iVar9 + 0x16c) = iVar12;
  do {
    if (uVar7 == 0) {
      uVar8 = *(uint *)(DAT_0000f700 + 0x20);
LAB_0000f5f4:
      uVar8 = (uVar8 & 0x7ffffff) >> 0x18;
    }
    else {
      if (uVar7 == 1) {
        uVar8 = *(uint *)(DAT_0000f700 + 0xa0);
        goto LAB_0000f5f4;
      }
      if (uVar7 == 2) {
        uVar8 = *(uint *)(DAT_0000f70c + 0x20);
        goto LAB_0000f5f4;
      }
      if (uVar7 == 3) {
        uVar8 = *(uint *)(DAT_0000f70c + 0xa0);
        goto LAB_0000f5f4;
      }
    }
    iVar3 = uVar7 * 0x6c + DAT_0000f714 + -0x80;
    uVar1 = (undefined1)uVar8;
    *(undefined1 *)(iVar3 + 0xa0) = uVar1;
    *(undefined1 *)(iVar3 + 0xa2) = uVar1;
    *(undefined1 *)(iVar3 + 0xa1) = uVar1;
    *(undefined1 *)(iVar3 + 0xa3) = 0;
    uVar4 = 0;
    iVar11 = uVar7 * 0x150 + DAT_0000f718;
    do {
      iVar9 = uVar4 * 0x54;
      iVar10 = uVar4 * 0x18;
      uVar4 = uVar4 + 1 & 0xff;
      *(int *)(iVar3 + iVar10 + 0xc0) = iVar11 + iVar9 + 0x7080;
    } while (uVar4 < 4);
    uVar7 = uVar7 + 1 & 0xff;
    if (3 < uVar7) {
      phy_set_band_reg(1);
      puVar2 = DAT_0000f71c;
      *DAT_0000f71c = DAT_0000f720;
      piVar5 = puVar2 + 1;
      bVar6 = 0;
      do {
        bVar6 = bVar6 + 1;
        *piVar5 = ((uint)puVar2 & 0x7fffff) + 0x22000000;
        piVar5 = piVar5 + 1;
      } while (bVar6 < 0x21);
      *piVar5 = -0x10000000;
      return;
    }
  } while( true );
}



/* ======================================================================
 * 0000f678  tsf_hw_init
 * ====================================================================== */

void tsf_hw_init(void)

{
  int iVar1;
  
  iVar1 = DAT_0000f6f8;
  *(undefined4 *)(DAT_0000f6f8 + -0x80) = DAT_0000f724;
  *(undefined4 *)(iVar1 + -0x7c) = DAT_0000f728;
  *(undefined4 *)(iVar1 + -0x78) = 0;
  *(undefined4 *)(iVar1 + -0x74) = 0;
  *(undefined4 *)(iVar1 + -0x70) = 0;
  *(undefined4 *)(iVar1 + -0x6c) = 0;
  *(undefined4 *)(iVar1 + -0x68) = 0;
  *(undefined4 *)(iVar1 + -100) = 0;
  *(undefined4 *)(iVar1 + -0x60) = 0;
  *(undefined4 *)(iVar1 + -0x5c) = 0;
  *(undefined4 *)(iVar1 + -0x58) = 0;
  *(undefined4 *)(iVar1 + -0x54) = 0;
  *(undefined4 *)(iVar1 + -0x4c) = 0;
  *(undefined4 *)(iVar1 + -0x48) = 0;
  *(undefined4 *)(iVar1 + -0x44) = 0;
  *(undefined4 *)(iVar1 + -0x4c) = 3;
  return;
}



/* ======================================================================
 * 0000f6a6  tsf_read
 * ====================================================================== */

int tsf_read(uint param_1)

{
  int iVar1;
  
  do {
  } while (*(int *)(DAT_0000f6f8 + -0x44) != *(int *)(DAT_0000f6f8 + -0x44));
  iVar1 = u64_add_u32(0,*(int *)(DAT_0000f6f8 + -0x44),*(undefined4 *)(DAT_0000f6f8 + -0x48));
  if (param_1 < 2) {
    iVar1 = iVar1 + *(int *)(param_1 * 0x98 + DAT_0000f72c + 0x488);
  }
  return iVar1;
}



/* ======================================================================
 * 0000f6d8  tsf_read_low
 * ====================================================================== */

int tsf_read_low(uint param_1)

{
  int iVar1;
  
  iVar1 = *(int *)(DAT_0000f6f8 + -0x48);
  if (param_1 < 2) {
    iVar1 = *(int *)(param_1 * 0x98 + DAT_0000f72c + 0x488) + iVar1;
  }
  return iVar1;
}



/* ======================================================================
 * 0000f730  mac_program_slot_timings
 * ====================================================================== */

void mac_program_slot_timings(int param_1,int param_2)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  iVar1 = DAT_0000f7a4;
  iVar3 = 10;
  if (param_1 << 0x1a < 0) {
    iVar3 = 0x10;
  }
  *(int *)(DAT_0000f7a4 + 0x24) = iVar3 + param_2;
  *(int *)(iVar1 + 0x28) = param_2 * 2 + iVar3;
  *(int *)(iVar1 + 0x20) = iVar3;
  *(int *)(iVar1 + 0x2c) = param_2 * 3 + iVar3;
  *(undefined4 *)(iVar1 + 0x30) = 2;
  *(int *)(iVar1 + 0x34) = param_2 * 8 + 2;
  *(int *)(iVar1 + 0x1c) = param_2;
  iVar4 = param_2 * 0x10 + 2;
  iVar3 = param_2 * 0x18 + 2;
  *(int *)(iVar1 + 0x3c) = iVar3;
  *(int *)(iVar1 + 0x38) = iVar4;
  *(int *)(DAT_0000f7a8 + 0x30) = param_2 * 8 + -1;
  iVar1 = DAT_0000f7a8;
  *(int *)(DAT_0000f7a8 + 0x58) = iVar4;
  *(int *)(iVar1 + 0x5c) = iVar3;
  iVar1 = DAT_0000f7ac;
  uVar2 = 0;
  do {
    **(undefined4 **)(iVar1 + uVar2 * 8) = *(undefined4 *)(uVar2 * 8 + iVar1 + 4);
    uVar2 = uVar2 + 1;
  } while (uVar2 < 0xb);
  *(undefined4 *)(DAT_0000f7b0 + 0x10) = 0;
  return;
}



/* ======================================================================
 * 0000f794  get_link_word_78
 * ====================================================================== */

int get_link_word_78(int param_1,int param_2)

{
  return (int)*(short *)(param_1 * 0x20 + DAT_0000f7ac + 0x78 + param_2 * 2);
}



/* ======================================================================
 * 0000f7b4  pac_phy_stop_op
 * ====================================================================== */

void pac_phy_stop_op(void)

{
  pac_phy_start_op(7);
  timer_cancel(DAT_0000fbb0);
  return;
}



/* ======================================================================
 * 0000f7c4  vif_set_basic_rates
 * ====================================================================== */

void vif_set_basic_rates(int param_1,undefined4 param_2)

{
  int iVar1;
  char *pcVar2;
  
  iVar1 = param_1 * 0x98 + DAT_0000fbb4;
  pcVar2 = (char *)(iVar1 + 0x470);
  *(undefined4 *)(iVar1 + 0x478) = param_2;
  if (*pcVar2 == '\x02') {
    pas_build_rate_tables(pcVar2);
    mac_program_slot_timings(*(undefined2 *)(DAT_0000fbb8 + 2),*(undefined4 *)(iVar1 + 0x4f8));
    pas_program_rate_tables(pcVar2);
    mac_program_ifs_timing();
  }
  return;
}



/* ======================================================================
 * 0000f7fc  phy_do_channel_switch
 * ====================================================================== */

void phy_do_channel_switch(undefined4 param_1)

{
  char cVar1;
  char *pcVar2;
  int iVar3;
  int iVar4;
  bool bVar5;
  
  pcVar2 = DAT_0000fbb8;
  if ((int)((uint)*(ushort *)(DAT_0000fbb8 + 2) << 0x1a) < 0) {
    iVar4 = 3;
  }
  else {
    iVar4 = 2;
    if ((~(uint)*(ushort *)(DAT_0000fbb8 + 2) & 0x11) != 0) {
      iVar4 = 0;
    }
  }
  DAT_0000fbb8[1] = '\0';
  phy_rx_disable_and_drain();
  iVar3 = DAT_0000fbbc;
  if (*(char *)(DAT_0000fbbc + 0x16) != '\x04') {
    thunk_16c92();
    pac_phy_start_op(0);
  }
  bVar5 = iVar4 != 3;
  cVar1 = *pcVar2;
  if (((cVar1 == '\x02') || (cVar1 == '\x04')) || (cVar1 == '\x01')) {
    bVar5 = false;
  }
  phy_cal_set_channel_and_arm(iVar4,param_1,bVar5);
  phy_temp_compensate_all_slots();
  pac_phy_start_op(1);
  phy_rx_enable();
  *(undefined1 *)(iVar3 + 0x16) = 4;
  evt_flags_set(DAT_0000fbc0,0x40000);
  return;
}



/* ======================================================================
 * 0000f872  mac_apply_channel_and_vif_config
 * ====================================================================== */

void mac_apply_channel_and_vif_config
               (char *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  char *pcVar1;
  int iVar2;
  undefined4 *puVar3;
  int iVar4;
  uint *puVar5;
  undefined4 uVar6;
  uint uVar7;
  int iVar8;
  uint uVar9;
  uint uVar10;
  
  *DAT_0000fbc4 = 0;
  pcVar1 = DAT_0000fbc8;
  if (*param_1 == '\x03') {
    DAT_0000fbc8[-0x98] = '\x01';
    param_1 = pcVar1;
  }
  else if (*param_1 != '\0') {
    fw_memcpy_words(DAT_0000fbc8,param_1,3,param_4,param_4);
  }
  pcVar1 = DAT_0000fbb8;
  *(undefined2 *)(DAT_0000fbb8 + 2) = *(undefined2 *)(param_1 + 2);
  *pcVar1 = *param_1;
  pcVar1[5] = param_1[1];
  iVar4 = DAT_0000fbd0;
  *(undefined4 *)(DAT_0000fbd0 + 0x24) = DAT_0000fbcc;
  *(undefined4 *)(iVar4 + 0x28) = 0;
  for (uVar10 = 0; iVar2 = DAT_0000fbbc, iVar8 = DAT_0000fbb4, uVar10 < (byte)param_1[6];
      uVar10 = uVar10 + 1) {
    uVar7 = (uint)(byte)param_1[uVar10 + 7];
    iVar8 = uVar7 * 0x98 + DAT_0000fbb4;
    *(undefined1 *)(iVar8 + 0x470) = 2;
    uVar9 = DAT_0000fbd8;
    if (*(char *)(iVar8 + 0x481) == *(char *)(iVar2 + 0x79)) {
      uVar9 = DAT_0000fbd4;
    }
    *(uint *)(iVar4 + 0x24) = *(uint *)(iVar4 + 0x24) | uVar9;
    if (*(char *)(iVar8 + 0x472) == '\x02') {
      *(uint *)(iVar4 + 0x24) = *(uint *)(iVar4 + 0x24) | 0x4000;
      *(int *)(iVar4 + 0x28) = iVar8 + 0x482;
      mac_program_bssid(iVar8 + 0x482,3);
    }
    if (((*(char *)(iVar8 + 0x472) == '\x06') || (*(char *)(iVar8 + 0x472) == '\x05')) &&
       (*(char *)(DAT_0000fbb4 + uVar7 + 0x46c) != '\0')) {
      *(uint *)(iVar4 + 0x24) = *(uint *)(iVar4 + 0x24) | 0x44000;
      puVar3 = DAT_0000fbc4;
      DAT_0000fbc4[0x16] = 0x2000000;
      puVar3[0x12] = &DAT_02000001;
    }
    pas_backoff_reset_all(uVar7);
  }
  *(undefined4 *)(DAT_0000fbb4 + 0xc) = 0;
  mac_reinit_after_wake();
  puVar5 = DAT_0000fbe0;
  uVar10 = DAT_0000fbdc & 0xf6ffffff;
  *(undefined1 *)(iVar4 + 0x1d) = 0;
  *puVar5 = uVar10;
  pcVar1 = DAT_0000fbb8;
  mac_program_slot_timings
            (*(undefined2 *)(DAT_0000fbb8 + 2),
             *(undefined4 *)((uint)(byte)param_1[7] * 0x98 + iVar8 + 0x4f8));
  pas_reprogram_all_vif_rate_tables();
  mac_program_ifs_timing();
  *(undefined2 *)(DAT_0000fbbc + 0x10) = *(undefined2 *)(param_1 + 4);
  phy_do_channel_switch(*(undefined2 *)(param_1 + 4));
  txp_build_ctl_frame(0,DAT_0000fbe4,0xd4);
  txp_build_ctl_frame(1,DAT_0000fbe4 + 0x54,0xd4);
  txp_build_ctl_frame(0,DAT_0000fbe4 + 0xa8,0xc4);
  txp_build_ctl_frame(1,DAT_0000fbe4 + 0xfc,0xc4);
  txp_install_resp_descs(3,0);
  *(undefined1 *)(DAT_0000fbd0 + -8) = 1;
  uVar6 = DAT_0000fbe8;
  txp_program_pipe_slot(DAT_0000fbe8,0x13,0,1,0);
  uVar10 = 0;
  do {
    if (*(char *)(iVar8 + uVar10 + 0x46c) != '\0') {
      txp_program_pipe_slot(uVar6,0x10,0,1,0);
      break;
    }
    uVar10 = uVar10 + 1;
  } while (uVar10 < 2);
  txp_program_pipe_slot(uVar6,8,0,1,0);
  txp_build_tbtt_desc(uVar6,1,1);
  mac_set_txop_limit(0);
  if (*param_1 == '\0') {
    mac_program_mode_sta();
  }
  else if (*param_1 == '\x05') {
    mac_program_mode_ap();
  }
  else {
    mac_program_mode_regs();
  }
  pcVar1[8] = '\0';
  pcVar1[9] = '\x10';
  return;
}



/* ======================================================================
 * 0000fa3c  link_activate
 * ====================================================================== */

void link_activate(int param_1)

{
  char cVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  int iVar7;
  uint uVar8;
  
  pas_backoff_reset_all();
  iVar3 = DAT_0000fbb4;
  *(undefined1 *)(param_1 * 0x98 + DAT_0000fbb4 + 0x470) = 2;
  pas_reprogram_all_vif_rate_tables();
  iVar4 = DAT_0000fbd0;
  *(undefined4 *)(DAT_0000fbd0 + 0x24) = DAT_0000fbcc;
  uVar5 = DAT_0000fbd4;
  cVar1 = *(char *)(iVar3 + 0x459);
  uVar6 = 0;
  uVar8 = DAT_0000fbd4 << 1;
  do {
    iVar7 = uVar6 * 0x98 + iVar3;
    if (*(char *)(iVar7 + 0x470) == '\x02') {
      uVar2 = uVar8;
      if (*(char *)(iVar7 + 0x481) == cVar1) {
        uVar2 = uVar5;
      }
      *(uint *)(iVar4 + 0x24) = *(uint *)(iVar4 + 0x24) | uVar2;
    }
    uVar6 = uVar6 + 1;
  } while (uVar6 < 3);
  return;
}



/* ======================================================================
 * 0000fa98  wait_pipes_idle
 * ====================================================================== */

void wait_pipes_idle(int param_1,uint param_2)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  uint uVar4;
  uint uVar5;
  undefined4 local_1c;
  
  iVar1 = fw_read_timer();
  uVar5 = 0;
  local_1c = param_1;
  if (param_2 != 0) {
    uVar5 = *(uint *)(DAT_0000fbec + 0xc);
  }
  do {
    uVar4 = 0;
    uVar2 = 0;
    do {
      if (*(char *)(uVar2 * 0x6c + DAT_0000fbb8 + 0xa3) != '\0') {
        uVar4 = uVar4 | 1 << uVar2;
      }
      uVar2 = uVar2 + 1 & 0xff;
    } while (uVar2 < 4);
    iVar3 = fw_read_timer();
    if (local_1c < iVar3 - iVar1) {
      iVar3 = fw_read_timer();
      iVar3 = func_0xfff019c8(4,iVar3 - iVar1);
      if (iVar3 == 0) {
        iVar1 = fw_read_timer();
      }
      if (1 < param_2) {
        txp_abort_all_pipes();
        iVar1 = txp_fn_4425(1,DAT_0000fbf0,0);
        if (iVar1 == 2) {
          mac_rx_restart();
        }
        goto LAB_0000fb3c;
      }
      if (param_2 != 0) {
        param_2 = param_2 + 1 & 0xff;
        local_1c = DAT_0000fbf4;
        iVar1 = fw_read_timer();
        if (param_2 != 0) {
          *(uint *)(DAT_0000fbec + 0xc) = uVar5 & 0xffffffc3;
        }
      }
    }
    if (uVar4 == 0) {
      if (param_2 != 0) {
LAB_0000fb3c:
        *(uint *)(DAT_0000fbec + 0xc) = uVar5;
      }
      return;
    }
  } while( true );
}



/* ======================================================================
 * 0000fb42  mac_radio_stop
 * ====================================================================== */

void mac_radio_stop(void)

{
  int iVar1;
  undefined4 uVar2;
  uint uVar3;
  uint uVar4;
  int iVar5;
  int iVar6;
  
  iVar1 = DAT_0000fbb8;
  *(undefined4 *)(DAT_0000fbb8 + 0x18) = 0;
  tx_flush_all_queues();
  uVar2 = irq_fiq_disable_save();
  iVar6 = DAT_0000fbf8;
  *(undefined2 *)(iVar1 + 8) = 0;
  *(undefined2 *)(iVar6 + 2) = 0;
  iVar6 = DAT_0000fbd0;
  *(undefined4 *)(DAT_0000fbd0 + -0x10) = 0;
  *(undefined1 *)(iVar6 + -8) = 4;
  mac_init_partial();
  iVar6 = DAT_0000fc00;
  uVar4 = DAT_0000fc00 + DAT_0000fbfc;
  uVar3 = 0;
  do {
    iVar5 = uVar3 * 4;
    uVar3 = uVar3 + 1;
    *(uint *)(iVar5 + iVar6 + 0x7000) = uVar4 & 0xf6ffffff;
  } while (uVar3 < 0x20);
  irq_fiq_restore(uVar2);
  phy_rx_disable_and_drain();
  pac_phy_stop_op();
  thunk_16c70();
  rx_handler_main_loop();
  iVar5 = DAT_0000fbd0;
  iVar6 = DAT_0000fbbc;
  *(undefined2 *)(DAT_0000fbbc + 0x10) = 0;
  *(undefined1 *)(iVar6 + 0x16) = 0;
  *(undefined1 *)(iVar5 + 0x1c) = 2;
  iVar6 = DAT_0000fdb0;
  *(undefined1 *)(iVar1 + 10) = 0;
  *(undefined1 *)(iVar1 + 0xb) = 0;
  *(undefined1 *)(iVar6 + 0x12) = 0;
  iVar1 = DAT_0000fdb4;
  uVar3 = 0;
  do {
    iVar6 = uVar3 * 0x98;
    uVar3 = uVar3 + 1;
    *(undefined1 *)(iVar6 + iVar1 + 0x470) = 1;
  } while (uVar3 < 3);
  evt_flags_clear(0x40000);
  return;
}



/* ======================================================================
 * 0000fc30  link_deactivate
 * ====================================================================== */

void link_deactivate(uint param_1)

{
  char cVar1;
  char *pcVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  longlong lVar7;
  
  iVar4 = DAT_0000fdb4;
  pcVar2 = (char *)(param_1 * 0x98 + DAT_0000fdb4 + 0x470);
  cVar1 = *pcVar2;
  *pcVar2 = '\x01';
  if ((param_1 < 2) && (*(char *)(iVar4 + param_1 + 0x46c) != '\0')) {
    uVar3 = 0;
    iVar5 = param_1 * 6 + iVar4;
    do {
      iVar6 = iVar5 + uVar3;
      uVar3 = uVar3 + 1;
      *(undefined1 *)(iVar6 + 0x460) = 0;
    } while (uVar3 < 6);
    *(undefined1 *)(iVar4 + param_1 + 0x46c) = 0;
    *(byte *)(iVar5 + 0x458) = *(byte *)(iVar5 + 0x458) ^ 0x80;
  }
  if (cVar1 == '\x02') {
    uVar3 = 0;
    do {
      if (*(char *)(uVar3 * 0x98 + iVar4 + 0x470) == '\x02') {
        pas_reprogram_all_vif_rate_tables();
        return;
      }
      uVar3 = uVar3 + 1;
    } while (uVar3 < 3);
    mac_radio_stop();
  }
  if (((uint)*(byte *)(DAT_0000fdb8 + 0x19) != (uint)*(byte *)(DAT_0000fdb8 + 0x1a)) &&
     (param_1 == *(byte *)(DAT_0000fdb8 + 0x19))) {
    iVar4 = (uint)*(byte *)(DAT_0000fdb8 + 0x1a) * 0x98 + iVar4;
    *(byte *)(iVar4 + 0x471) = *(byte *)(iVar4 + 0x471) & 0xf7;
    lVar7 = tsf_read(2);
    lVar7 = lVar7 + *(longlong *)(iVar4 + 0x488);
    tsf_write((int)lVar7,(int)((ulonglong)lVar7 >> 0x20));
    return;
  }
  return;
}



/* ======================================================================
 * 0000fcde  mac_reset_and_drain
 * ====================================================================== */

void mac_reset_and_drain(void)

{
  undefined4 uVar1;
  
  tx_flush_all_queues();
  uVar1 = irq_fiq_disable_save();
  mac_init_partial();
  *(undefined4 *)(DAT_0000fdc0 + 0x30) = DAT_0000fdbc;
  irq_fiq_restore(uVar1);
  phy_rx_disable_and_drain();
  rx_handler_main_loop();
  return;
}



/* ======================================================================
 * 0000fd04  link_slot_alloc
 * ====================================================================== */

/* link_slot_alloc() -- allocate an INTERNAL link slot.  Returns 0..9.
   
     for (i = 0; i < 10; i++)
         if (!(bitmap & (1 << i))) {
             bitmap |= 1 << i;
             if (i != 9) zero 0x10 u16 at 0x000049E0 + i*0x20 + g_vif[0].. + 0x18;
             return i;
         }
     return 9;                       /* exhausted */
   
   Bitmap is the u16 at 0x040089D0 (= DAT_0000FDC4 + 0x18).  Freed by
   link_slot_free (0x0000FD4C), which only accepts id < 10.
   
   *** THIS IS A SEPARATE BITMAP from the per-link state gate at 0x04003E90
   *** (= g_vif_hdr + 0x18 = g_fw_ctx + 0x818) used by
   *** txq_try_append_to_aggregate and link_states_reset_to_1.  They are easy to
   *** conflate; verified distinct.
   
   CAPACITY: only 10 slots, and index 9 doubles as the "exhausted" return AND
   is special-cased out of the per-slot clear -- so 9 looks like a reserved
   overflow/aliasing slot and the usable count is 0..8 = NINE links.
   
   Called by ap_map_link, which stores the result at vif+0x1A.  Note ap_map_link
   separately bounds the HOST link_id to < 0x0F with a 16-entry map, so there are
   two distinct id spaces and the internal one is the smaller.  This lowers the
   practical AP station ceiling below the ~13 previously derived from the host
   side alone -- and because exhaustion returns a valid-looking 9 rather than an
   error, the failure mode is silent aliasing onto slot 9 rather than a clean
   rejection. */

uint link_slot_alloc(void)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  
  uVar2 = 0;
  do {
    if ((1 << (uVar2 & 0xff) & (uint)*(ushort *)(DAT_0000fdc4 + 0x18)) == 0) {
      uVar2 = uVar2 & 0xff;
      *(ushort *)(DAT_0000fdc4 + 0x18) = *(ushort *)(DAT_0000fdc4 + 0x18) | (ushort)(1 << uVar2);
      iVar1 = DAT_0000fdc8;
      if (uVar2 != 9) {
        iVar4 = uVar2 * 0x20 + DAT_0000fdcc;
        iVar3 = 0;
        do {
          iVar5 = iVar3 * 2;
          iVar3 = iVar3 + 1;
          *(undefined2 *)(iVar4 + iVar5 + iVar1 + 0x18) = 0;
        } while (iVar3 < 0x10);
      }
      return uVar2;
    }
    uVar2 = uVar2 + 1;
  } while ((int)uVar2 < 10);
  return 9;
}



/* ======================================================================
 * 0000fd4c  link_slot_free
 * ====================================================================== */

void link_slot_free(uint param_1)

{
  if (param_1 < 10) {
    *(ushort *)(DAT_0000fdc4 + 0x18) =
         *(ushort *)(DAT_0000fdc4 + 0x18) & ~(ushort)(1 << (param_1 & 0xff));
  }
  return;
}



/* ======================================================================
 * 0000fd5e  phy_set_band_reg
 * ====================================================================== */

void phy_set_band_reg(int param_1)

{
  undefined4 uVar1;
  
  if (param_1 == 1) {
    uVar1 = 0xbf;
  }
  else {
    uVar1 = 0xab;
  }
  *(undefined4 *)(DAT_0000fdd0 + 0xc) = uVar1;
  return;
}



/* ======================================================================
 * 0000fd6e  hif_dbg_ctx_init
 * ====================================================================== */

void hif_dbg_ctx_init(void)

{
  undefined4 *puVar1;
  
  puVar1 = DAT_0000fdd4;
  DAT_0000fdd4[1] = 0;
  puVar1[2] = 0;
  puVar1[3] = DAT_0000fdd8;
  puVar1[4] = 0;
  *puVar1 = 0;
  return;
}



/* ======================================================================
 * 0000fd80  hif_dbg_ctx_set
 * ====================================================================== */

void hif_dbg_ctx_set(ushort *param_1)

{
  ushort uVar1;
  undefined4 *puVar2;
  
  puVar2 = DAT_0000fdd4;
  uVar1 = param_1[2];
  DAT_0000fdd4[1] = (uint)*param_1 + (uint)param_1[1] * 0x10000;
  puVar2[2] = (uint)uVar1;
  puVar2[3] = DAT_0000fdd8;
  puVar2[4] = 0;
  *puVar2 = 0x10;
  return;
}



/* ======================================================================
 * 0000fddc  mac_program_ifs_timing
 * ====================================================================== */

void mac_program_ifs_timing(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  if ((~*(ushort *)(DAT_0000fff8 + 2) & 0x11) == 0) {
    iVar1 = 1;
    iVar2 = 0;
  }
  else {
    iVar1 = 2;
    iVar2 = 0xb;
  }
  uVar3 = (uint)*(byte *)(DAT_0000fffc + iVar1 * 4) + iVar2 & 0xff;
  if (iVar1 == 2) {
    uVar3 = uVar3 - 8 & 0xff;
  }
  *(uint *)(DAT_0000fff8 + 0x44) =
       (*(int *)(DAT_0000fff8 + 0x1c) * 3 +
       (*(uint *)(uVar3 * 0x10 + DAT_00010000 + DAT_00010004) & 0xffffff)) * 8;
  return;
}



/* ======================================================================
 * 0000fe26  pas_rate_retries_for_idx
 * ====================================================================== */

byte pas_rate_retries_for_idx(int param_1,uint param_2)

{
  return *(byte *)((param_2 >> 1) + param_1 + 8) >> ((param_2 & 1) << 2) & 0xf;
}



/* ======================================================================
 * 0000fe38  pas_highest_rate_in_policy
 * ====================================================================== */

char pas_highest_rate_in_policy(undefined4 param_1)

{
  int iVar1;
  char cVar2;
  
  for (cVar2 = '\x15';
      (iVar1 = pas_rate_retries_for_idx(param_1,cVar2), iVar1 == 0 && (cVar2 != -1));
      cVar2 = cVar2 + -1) {
  }
  if (cVar2 == -1) {
    fw_assert(s_pas_rates_c_00010008,0x18d,9);
  }
  return cVar2;
}



/* ======================================================================
 * 0000fe6a  phy_build_rate_cfg
 * ====================================================================== */

/* phy_build_rate_cfg(band, rate_mask_IGNORED, preamble, chan_flags)
   Builds the u16 stored at vif+0x20 (phy_rate_cfg).
   
     r0 = (band == 1) ? 0x26 : 0x17
     preamble 0 -> |= 0x100,  1 -> |= 0x200,  2 -> |= 0x400
     if (chan_flags & 0x100)  |= 0x40
     return r0
   
   *** THE SECOND ARGUMENT IS NEVER READ. *** Verified at instruction level
   (0x0000FE6A..0x0000FE98): r1 is overwritten as scratch at 0xFE84/0xFE8A/
   0xFE94 and never read. Both call sites --
     FUN_00005028      (the WSM_START commit path)
     phy_recompute_rate_cfg (0x0001524E)
   pass `*(u32 *)(0x04008608) | vif->basic_rate_set` as that argument, and it
   is discarded.
   
   CONSEQUENCE: the only two identified consumers of vif->basic_rate_set
   (+0x28) feed it into a function that ignores it, so this path has no effect.
   That does not prove basic_rate_set is unused everywhere -- other readers may
   exist and have not been ruled out -- but it does mean the previously
   documented "basic_rate_set derives the PHY rate config" claim is wrong, and
   the case for the empty-basic-rate-set driver fix is correspondingly weaker:
   correct on mac80211 grounds, but with no demonstrated functional effect. */

uint phy_build_rate_cfg(int param_1,undefined4 param_2,int param_3,int param_4)

{
  uint uVar1;
  uint uVar2;
  
  if (param_1 == 1) {
    uVar1 = 0x26;
  }
  else {
    uVar1 = 0x17;
  }
  if (param_3 == 0) {
    uVar2 = 0x100;
  }
  else if (param_3 == 1) {
    uVar2 = 0x200;
  }
  else {
    if (param_3 != 2) goto LAB_0000fe90;
    uVar2 = 0x400;
  }
  uVar1 = uVar1 | uVar2;
LAB_0000fe90:
  if (param_4 << 0x17 < 0) {
    uVar1 = uVar1 | 0x40;
  }
  return uVar1;
}



/* ======================================================================
 * 0000fe9a  phy_temp_compensate_all_slots
 * ====================================================================== */

/* phy_temp_compensate_all_slots() -- recompute the temperature-compensated TX gain
   for all 16 channel slots.
   
     for (slot = 0; slot < 0x10; slot++)
         phy_temp_compensate(slot, slot, (s16)g_phy_ctx[0x44]);   /* 0x44 = die temp */
   
   *** CANDIDATE FOR THE THROUGHPUT SYMPTOM -- INVESTIGATED AND RULED OUT. ***
   This looked like a strong lead: the firmware genuinely does have a die-temperature
   sensor and a temperature-driven TX-gain correction, which is exactly the shape of
   "MCS holds but retries climb monotonically over a long run".  DRIVER-API-NOTES.md
   also records TPA (0x000C / 0x1041) as absent from this build, so a *separate*
   thermal loop would have been a real find.
   
   But it is not periodic.  The call graph is:
   
     phy_do_channel_switch (0x0000F7FC) --> here
     0x00013CAE                         --> here
     rf_measure_temp_and_vbat (0x00019FE2) <-- phy_set_channel_full, phy_state_cmd_dispatch
     rf_latch_temp_readings   (0x0001A140) <-- phy_state_cmd_dispatch
   
   Every entry point is **channel-change or calibration-state driven**.  The PHY
   calibration state machine (phy_cal_step_start/_measure/_done) is kicked from
   phy_do_channel_switch and then chains through its own timers until it completes;
   nothing re-arms it on a periodic tick while traffic is flowing.
   
   **Conclusion: temperature compensation is computed at channel switch / association
   and then stays fixed.** On a fixed-channel bulk transfer it cannot drift, so it
   does not explain within-run or run-to-run degradation.  Recorded so this lead is
   not chased again.
   
   Temperature sensing itself is real and readable if ever wanted:
   phy_compute_temp_from_adc (0x000199A8) derives a temperature and a x100 scaled
   value from three ADC taps, and rf_measure_temp_and_vbat converts two mux settings
   via (raw*0x47 - offset)*1000/slope, storing results at g_phy_ctx+0x48 and +0x4C. */

void phy_temp_compensate_all_slots(void)

{
  int iVar1;
  byte bVar2;
  
  iVar1 = DAT_00010014;
  bVar2 = 0;
  do {
    phy_temp_compensate(bVar2,bVar2,(int)(short)*(undefined4 *)(iVar1 + 0x44));
    bVar2 = bVar2 + 1;
  } while (bVar2 < 0x10);
  return;
}



/* ======================================================================
 * 0000feba  pas_install_tx_rate_policy
 * ====================================================================== */

/* pas_install_tx_rate_policy(struct wsm_tx_rate_retry_policy_entry *e)
   pas_rates.c -- installs one 20-byte TX rate retry policy.
   
   Host entry layout (matches cw1200 struct wsm_tx_rate_retry_policy exactly):
     +0 index, +1 short_retries, +2 long_retries, +3 flags,
     +4 rate_recoveries, +5..7 reserved, +8..19 rate_count_indices[3]
   
     memcpy(g_fw_ctx + 0xF0 + index*0x14, e, 0x14);
     state = g_fw_ctx + index*4 + 0x370;
     state[0..3] = 0;
     if ((e->flags & 3) == 1 || (e->flags & 3) == 2) {
         hi = pas_highest_rate_in_policy(e);       // scans idx 0x15..0 = MCS7..
         state[0] = hi;                            // highest rate in policy
         state[1] = hi;                            // current rate
         state[2] = pas_rate_retries_for_idx(e, hi);
         if (e->flags & BIT(3))  state[2]--;       // COUNT_INITIAL_TRANSMIT
     }
   
   rate_count_indices is 24 packed nibbles, one retry count per rate index
   0..21 (0x15 = MCS7), read by pas_rate_retries_for_idx.
   
   IMPORTANT for the TX-retry investigation: both drivers set
   flags = BIT(2)|BIT(3) = 0x0C, so `flags & 3` is 0 and this cached
   rate-walk state is NEVER initialised here -- state[0..3] stay zero and the
   BIT(3) decrement is unreachable.  This is identical for mainline and both
   vendor trees, so it is not a mainline regression.  Whether the walk is
   instead recomputed per-transmission, or whether modes 1/2 are the intended
   configuration and 0 is a degraded path, is UNRESOLVED -- trace the readers
   of g_fw_ctx+0x370 before drawing any conclusion. */

void pas_install_tx_rate_policy(byte *param_1)

{
  char cVar1;
  undefined4 uVar2;
  int iVar3;
  
  iVar3 = (uint)*param_1 * 4 + DAT_00010018;
  fw_memcpy((void *)((uint)*param_1 * 0x14 + DAT_00010018 + 0xf0),param_1,0x14);
  *(undefined1 *)(iVar3 + 0x370) = 0;
  *(undefined1 *)(iVar3 + 0x371) = 0;
  *(undefined1 *)(iVar3 + 0x372) = 0;
  *(undefined1 *)(iVar3 + 0x373) = 0;
  if (((param_1[3] & 3) == 1) || ((param_1[3] & 3) == 2)) {
    uVar2 = pas_highest_rate_in_policy(param_1);
    *(undefined1 *)(iVar3 + 0x370) = (char)uVar2;
    *(char *)(iVar3 + 0x371) = (char)uVar2;
    cVar1 = pas_rate_retries_for_idx(param_1,uVar2);
    *(char *)(iVar3 + 0x372) = cVar1;
    if ((int)((uint)param_1[3] << 0x1c) < 0) {
      *(char *)(iVar3 + 0x372) = cVar1 + -1;
    }
  }
  return;
}



/* ======================================================================
 * 0000ff14  pas_next_lower_rate
 * ====================================================================== */

uint pas_next_lower_rate(undefined4 param_1,uint param_2,undefined1 *param_3)

{
  int iVar1;
  
  if (param_2 == 0) {
    return 0xff;
  }
  do {
    param_2 = param_2 - 1 & 0xff;
    if (0xfe < param_2) {
      return param_2;
    }
    iVar1 = pas_rate_retries_for_idx(param_1,param_2);
    *param_3 = (char)iVar1;
  } while (iVar1 == 0);
  return param_2;
}



/* ======================================================================
 * 0000ff46  tx_bump_try_count
 * ====================================================================== */

void tx_bump_try_count(int param_1)

{
  if (*(int *)(param_1 + 4) << 0x1a < 0) {
    do {
      *(short *)(param_1 + 0x1e) = *(short *)(param_1 + 0x1e) + 1;
      param_1 = *(int *)(param_1 + 0x3c);
    } while (param_1 != 0);
    return;
  }
  *(short *)(param_1 + 0x1e) = *(short *)(param_1 + 0x1e) + 1;
  return;
}



/* ======================================================================
 * 0000ff62  pas_backoff_cw_update
 * ====================================================================== */

/* pas_backoff_cw_update(tx_ctx) -- contention-window growth, pas_rates.c.
   
     base  = g_fw_ctx + link*0x98            (link = tx_ctx[0x69])
     ac    = tx_ctx[0x0C]
     cw    = base + 0x4BC + ac*4             (0x4CC-0x10)
     tries = base + ac*4 + 0x4AC
   
     if ((*tries & 1) == 0)  *cw = *cw * 2 + 1;   // doubles every OTHER retry
     if (g_backoff_ctrl[0] == 0)
         clamp *cw to the per-AC CWmax at base + 0x4CC + 8 + ac*2
     else
         clamp *cw to g_backoff_ctrl[2]
     (*tries)++;
   
   g_backoff_ctrl (0x04002088) is written by MIB 0x1039 BACKOFF_CTRL, whose
   handler stores 3 words and clamps the third to 0x400.  So the host can
   override the contention-window ceiling for every AC at once, up to 1024.
   The vendor driver exports wsm_set_backoff_ctrl(); mainline has nothing.
   
   Note the `& 1` test: CW only doubles on even retry counts, i.e. the window
   grows half as fast as textbook binary exponential backoff. */

void pas_backoff_cw_update(int param_1)

{
  byte bVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  uint *puVar5;
  int iVar6;
  
  bVar1 = *(byte *)(param_1 + 0xc);
  iVar4 = (uint)*(byte *)(param_1 + 0x69) * 0x98 + DAT_00010018;
  iVar6 = iVar4 + DAT_0001001c;
  iVar3 = (uint)bVar1 * 4;
  iVar4 = iVar4 + iVar3;
  puVar5 = (uint *)(iVar4 + DAT_0001001c + -0x10);
  if ((*(uint *)(iVar4 + 0x4ac) & 1) == 0) {
    *puVar5 = *puVar5 * 2 + 1;
  }
  if (*DAT_00010020 == 0) {
    uVar2 = (uint)*(ushort *)((uint)bVar1 * 2 + iVar6 + 8);
    if (uVar2 < *puVar5) {
      *puVar5 = uVar2;
    }
  }
  else if ((uint)DAT_00010020[2] < *puVar5) {
    *puVar5 = DAT_00010020[2];
  }
  iVar3 = (uint)*(byte *)(param_1 + 0x69) * 0x98 + DAT_00010018 + iVar3;
  *(int *)(iVar3 + 0x4ac) = *(int *)(iVar3 + 0x4ac) + 1;
  return;
}



/* ======================================================================
 * 0000ffcc  pas_policy_retime_if_mode2
 * ====================================================================== */

void pas_policy_retime_if_mode2(xr_tx_pas *param_1)

{
  if ((param_1->bRatePolicyIdx != 0xf) &&
     ((*(byte *)((uint)param_1->bRatePolicyIdx * 0x14 + DAT_00010018 + 0xf3) & 3) == 2)) {
    pas_tx_policy_prepare(param_1);
    pas_compute_tx_timing(param_1);
  }
  return;
}



/* ======================================================================
 * 00010024  sync_regs_10
 * ====================================================================== */

void sync_regs_10(void)

{
  undefined4 *puVar1;
  int iVar2;
  undefined4 uVar3;
  
  iVar2 = DAT_0001003c;
  puVar1 = DAT_00010038;
  uVar3 = DAT_00010038[1];
  *(undefined4 *)(DAT_0001003c + 0x10) = uVar3;
  *(undefined4 *)(iVar2 + 0x14) = uVar3;
  puVar1[2] = uVar3;
  *puVar1 = *puVar1;
  return;
}



/* ======================================================================
 * 00010040  txp_build_ctl_frame
 * ====================================================================== */

void txp_build_ctl_frame(int param_1,uint *param_2,int param_3,undefined4 param_4)

{
  int iVar1;
  
  iVar1 = 0x15;
  if (param_3 != 0xd4) {
    iVar1 = 9;
  }
  *param_2 = *(int *)(DAT_000100b8 + 0x30) << 0x10 | 0x20000000;
  *(undefined2 *)(param_2 + 1) = 0;
  *(undefined2 *)((int)param_2 + 6) = 0;
  param_2[2] = 0x58000000;
  if (param_3 == 0xd4) {
    txp_build_ack_desc(param_1);
  }
  else {
    txp_build_cts_desc(param_1,param_2 + 3,param_3,param_4,param_4);
  }
  txp_program_pipe_slot(param_2,iVar1 + param_1,1,1,0);
  if ((param_3 == 0xd4) && (*(char *)(DAT_000100bc + param_1 + 0x46c) != '\0')) {
    txp_program_pipe_slot(param_2,0x14,1,1,0);
  }
  return;
}



/* ======================================================================
 * 000100c0  pas_backoff_reset_all
 * ====================================================================== */

void pas_backoff_reset_all(undefined4 param_1)

{
  byte bVar1;
  
  bVar1 = 0;
  do {
    pas_backoff_reset(param_1,bVar1);
    bVar1 = bVar1 + 1;
  } while (bVar1 < 4);
  return;
}



/* ======================================================================
 * 000100dc  txp_program_duration
 * ====================================================================== */

void txp_program_duration(int param_1,undefined4 param_2,undefined4 param_3,uint param_4)

{
  int iVar1;
  uint uVar2;
  uint local_10;
  
  local_10 = param_4;
  if (*(int *)(DAT_000101b0 + 0x28) == 0) {
    *(undefined4 *)(DAT_000101b0 + 0x28) = 1;
    *(undefined1 *)(DAT_000101b4 + 0x19) = 0;
    if (*(char *)(param_1 * 0x3b0 + DAT_000101b8 + 0x18) != '\x06') {
      txp_program_pipe_slot(DAT_000101bc,0x10,1,1,1,param_3);
    }
    *DAT_000101c4 = DAT_000101c0 & 0xf6ffffff;
  }
  iVar1 = DAT_000101cc;
  airtime_compute(&local_10,0,*(undefined1 *)(param_1 * 0x70 + DAT_000101c8 + 0x1f),
                  *(undefined2 *)(DAT_000101cc + 2),0x18,1);
  uVar2 = (uint)*(ushort *)(iVar1 + 2);
  if (((int)(uVar2 << 0x1e) < 0) && ((int)(uVar2 << 0x1b) < 0)) {
    local_10 = (local_10 & 0xffff) - 6;
  }
  else {
    if (((*(ushort *)(iVar1 + 2) & 1) == 0) && ((int)(uVar2 << 0x19) < 0)) {
      local_10 = (local_10 & 0xffff) + 3;
      goto LAB_00010160;
    }
    local_10 = local_10 & 0xffff;
  }
  local_10 = local_10 + 1;
LAB_00010160:
  *(uint *)(DAT_000101d0 + 0x14) = local_10 & 0xffff;
  return;
}



/* ======================================================================
 * 00010172  mac_disable_beacon_hw
 * ====================================================================== */

void mac_disable_beacon_hw(uint param_1,int param_2)

{
  int iVar1;
  int iVar2;
  uint *puVar3;
  uint uVar4;
  
  iVar2 = DAT_000101b4;
  iVar1 = DAT_000101b0;
  if (2 < *(uint *)(DAT_000101b0 + 0x28)) {
    if (*(byte *)(DAT_000101b4 + 0x18) == param_1) {
      *(undefined4 *)(DAT_000101d0 + -0x78) = 0;
      *(undefined1 *)(iVar2 + 0x19) = 0;
    }
  }
  if (param_2 != 0) {
    irq_fiq_disable_save();
    puVar3 = DAT_000101d4;
    uVar4 = *(uint *)(iVar1 + 0x30) & 0xfffffffe;
    *(uint *)(iVar1 + 0x30) = uVar4;
    *puVar3 = uVar4;
    irq_fiq_restore();
    *(undefined4 *)(iVar1 + 0x28) = 0;
    return;
  }
  *(undefined4 *)(iVar1 + 0x28) = 1;
  return;
}



/* ======================================================================
 * 000101d8  tx_requeue
 * ====================================================================== */

void tx_requeue(int param_1)

{
  *(undefined1 *)(param_1 + 0x53) = 0;
  *(uint *)(param_1 + 0x2c) = *(uint *)(param_1 + 0x2c) | 0x80000;
  pas_txq_compact_and_push(DAT_0001051c,param_1);
  return;
}



/* ======================================================================
 * 000101f4  txp_fn_4155
 * ====================================================================== */

void txp_fn_4155(void)

{
  byte bVar1;
  int iVar2;
  uint *puVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  int iVar7;
  byte *pbVar8;
  uint uVar9;
  int iVar10;
  byte *pbVar11;
  uint uVar12;
  int iVar13;
  uint local_20;
  
  iVar2 = DAT_0001051c;
  if (-1 < (int)((uint)*(byte *)(DAT_0001051c + -4) << 0x1d)) {
    return;
  }
  local_20 = 0;
LAB_00010208:
  iVar10 = local_20 * 0x6c + DAT_00010520;
  pbVar11 = (byte *)(iVar10 + 0xa0);
  uVar12 = (uint)*(byte *)(iVar10 + 0xa2);
  iVar4 = fw_read_timer();
  iVar5 = txp_pipe_advance_slot(local_20);
  if (iVar5 != 0) {
    do {
      pbVar8 = pbVar11 + uVar12 * 0x18 + 0xc;
      iVar5 = *(int *)(pbVar11 + uVar12 * 0x18 + 0x18);
      if (*pbVar8 == 1) {
        if (iVar5 != 0) {
          bVar1 = *(byte *)(iVar5 + 0x6c);
          link_set_state((uint)bVar1,5);
          uVar6 = 0;
          iVar13 = (uint)bVar1 * 0x40 + DAT_00010520;
          do {
            iVar7 = uVar6 * 4;
            uVar6 = uVar6 + 1 & 0xff;
            *(undefined4 *)(iVar13 + iVar7 + 0x490) = 0;
          } while (uVar6 < 0x10);
          if (*pbVar8 != 1) goto LAB_00010274;
          goto LAB_000102b2;
        }
LAB_000102bc:
        if (*(int *)(pbVar11 + uVar12 * 0x18 + 0x1c) != 0) {
          desc_freelist_push();
        }
      }
      else {
LAB_00010274:
        if (*pbVar8 == 0) {
LAB_000102b2:
          for (; iVar5 != 0; iVar5 = *(int *)(iVar5 + 0x3c)) {
            if ((*(int *)(iVar5 + -0x14) - iVar4) + DAT_00010524 < 0) {
              *(undefined2 *)(iVar5 + 0x1c) = 0x18;
              tx_ctx_free_inner(iVar5);
            }
            else {
              *(undefined2 *)(iVar5 + 0x50) = 0;
              *(uint *)(iVar5 + 4) = *(uint *)(iVar5 + 4) & DAT_00010528;
              *(undefined2 *)(iVar5 + 0x1c) = 0xfe;
              tx_requeue(iVar5);
            }
          }
          if (*pbVar8 == 1) goto LAB_000102bc;
        }
      }
      pbVar8 = pbVar11 + uVar12 * 0x18 + 0x18;
      pbVar8[0] = 0;
      pbVar8[1] = 0;
      pbVar8[2] = 0;
      pbVar8[3] = 0;
      pbVar11[uVar12 * 0x18 + 0xf] = 0;
      if (*(byte *)(iVar10 + 0xa1) == uVar12) goto LAB_000102da;
      uVar12 = uVar12 + 1 & 3;
    } while( true );
  }
  goto LAB_000102e6;
LAB_000102da:
  *(char *)(iVar10 + 0xa2) = *(char *)(iVar10 + 0xa1);
  *pbVar11 = *(char *)(iVar10 + 0xa1) + 1U & 3;
LAB_000102e6:
  uVar12 = *(uint *)(*(int *)(iVar10 + 0xa8) + 0x20);
  if ((uVar12 & 0x7ffffff) >> 0x18 != (uVar12 & 0x3fffffff) >> 0x1b) {
    fw_assert(s_tx_ptcs_c_00010534,DAT_00010530,DAT_0001052c);
  }
  *(byte *)(iVar10 + 0xa4) = *(byte *)(iVar10 + 0xa4) & 0xf6;
  *(undefined1 *)(iVar10 + 0xa5) = 5;
  puVar3 = DAT_00010540;
  local_20 = local_20 + 1 & 0xff;
  if (3 < local_20) {
    uVar6 = DAT_00010540[1];
    uVar9 = *DAT_00010540 & 0xff;
    for (uVar12 = uVar9; uVar12 != (uVar6 & 0xff); uVar12 = uVar12 + 1 & 0x3f) {
      if (puVar3[uVar12 + 2] != 0) {
        tx_requeue();
        puVar3[uVar12 + 2] = 0;
      }
    }
    while ((uVar9 != (uVar6 & 0xff) && (puVar3[uVar9 + 2] == 0))) {
      uVar9 = uVar9 + 1 & 0x3f;
      *puVar3 = uVar9;
    }
    *(byte *)(iVar2 + -4) = *(byte *)(iVar2 + -4) ^ 4 | 8;
    evt_flags_set(DAT_00010544,0x80000000);
    return;
  }
  goto LAB_00010208;
}



/* ======================================================================
 * 00010388  mac_init_partial
 * ====================================================================== */

void mac_init_partial(void)

{
  undefined4 *puVar1;
  
  mac_noop();
  puVar1 = DAT_00010548;
  DAT_00010548[10] = 0;
  *puVar1 = DAT_0001054c;
  puVar1[1] = *(undefined4 *)(DAT_00010550 + 0x34);
  return;
}



/* ======================================================================
 * 000103a0  txp_program_pipe_slot
 * ====================================================================== */

void txp_program_pipe_slot(uint param_1,int param_2,int param_3,int param_4,int param_5)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  
  if (0x1a < param_2 - 2U) {
    fw_assert(s_tx_ptcs_c_00010534,DAT_00010554,0x33);
  }
  *(uint *)(param_2 * 4 + DAT_00010558 + 0x7000) = param_1 & 0xf6ffffff;
  iVar1 = DAT_00010548;
  if (param_2 == 0x1c) {
    uVar2 = 0x1f;
  }
  else if (param_2 == 0x1b) {
    uVar2 = 0x1e;
  }
  else {
    uVar2 = param_2 - 2;
  }
  if (param_3 == 0) {
    uVar3 = 0x1f;
    if (uVar2 != 0x1e) {
      uVar3 = uVar2;
    }
    *(uint *)(DAT_00010548 + 8) = *(uint *)(DAT_00010548 + 8) & ~(1 << (uVar3 & 0xff));
  }
  else {
    uVar3 = 0x1f;
    if (uVar2 != 0x1e) {
      uVar3 = uVar2;
    }
    *(uint *)(DAT_00010548 + 8) = 1 << (uVar3 & 0xff) | *(uint *)(DAT_00010548 + 8);
  }
  if (param_4 == 0) {
    uVar3 = 0x1f;
    if (uVar2 != 0x1e) {
      uVar3 = uVar2;
    }
    uVar3 = *(uint *)(iVar1 + 0xc) & ~(1 << (uVar3 & 0xff));
  }
  else {
    uVar3 = 0x1f;
    if (uVar2 != 0x1e) {
      uVar3 = uVar2;
    }
    uVar3 = *(uint *)(iVar1 + 0xc) | 1 << (uVar3 & 0xff);
  }
  *(uint *)(iVar1 + 0xc) = uVar3;
  if (param_5 == 0) {
    uVar2 = *(uint *)(iVar1 + 0x10) & ~(1 << (uVar2 & 0xff));
  }
  else {
    uVar2 = 1 << (uVar2 & 0xff) | *(uint *)(iVar1 + 0x10);
  }
  *(uint *)(iVar1 + 0x10) = uVar2 & 0xffffff;
  return;
}



/* ======================================================================
 * 00010456  txp_prepare_all_pipes_idle
 * ====================================================================== */

void txp_prepare_all_pipes_idle(void)

{
  int iVar1;
  uint *flags;
  uint uVar2;
  uint uVar3;
  
  iVar1 = DAT_0001051c;
  *(byte *)(DAT_0001055c + 0x15) = *(byte *)(DAT_0001055c + 0x15) | 2;
  uVar3 = 0;
  if (*(char *)(iVar1 + -4) != '\0') {
    fw_assert(s_tx_ptcs_c_00010534,DAT_00010530 - 0x97,DAT_00010560);
  }
  uVar2 = 0;
  do {
    if (*(char *)(uVar2 * 0x6c + DAT_00010520 + 0xa3) == '\0') {
      uVar3 = 1 << uVar2 & 0xffU | uVar3;
    }
    uVar2 = uVar2 + 1 & 0xff;
  } while (uVar2 < 4);
  if (uVar3 == 0xf) {
    *(byte *)(iVar1 + -4) = *(byte *)(iVar1 + -4) | 4;
    txp_fn_4155();
    return;
  }
  txp_abort_all_pipes();
  flags = DAT_00010544;
  *(undefined1 *)(iVar1 + -4) = 2;
  evt_flags_set(flags,0x200000);
  return;
}



/* ======================================================================
 * 000104c6  txp_dequeue_pending
 * ====================================================================== */

undefined4 txp_dequeue_pending(uint *param_1)

{
  uint *puVar1;
  uint *puVar2;
  uint uVar3;
  uint uVar4;
  undefined4 local_18;
  
  puVar1 = DAT_0001051c;
  local_18 = 0;
  *param_1 = 0;
  puVar2 = DAT_0001051c;
  if ((int)((uint)(byte)puVar1[-1] << 0x1c) < 0) {
    uVar4 = *DAT_0001051c & 0xff;
    if (uVar4 == (DAT_0001051c[1] & 0xff)) {
      *(undefined1 *)(puVar1 + -1) = 0;
    }
    else {
      uVar3 = DAT_0001051c[uVar4 + 2];
      *param_1 = uVar3;
      if (uVar3 == 0) {
        fw_assert(s_tx_ptcs_c_00010534,DAT_00010568,DAT_00010564);
      }
      puVar2[uVar4 + 2] = 0;
      *puVar2 = uVar4 + 1 & 0x3f;
    }
  }
  else {
    local_18 = 1;
  }
  return local_18;
}



/* ======================================================================
 * 0001056c  txp_build_ack_desc
 * ====================================================================== */

void txp_build_ack_desc(int param_1,int *param_2)

{
  *param_2 = (uint)*(byte *)(DAT_000108f0 + 0x18) * 0x2000 + 0x59000000;
  param_2[1] = DAT_000108f4;
  param_2[2] = DAT_000108f8;
  param_2[3] = DAT_000108fc;
  param_2[4] = 0x47000000;
  param_2[5] = (DAT_00010900 + param_1 + DAT_00010904 & 0x7fffffU) + 0x20800000;
  param_2[6] = DAT_00010908;
  param_2[7] = DAT_0001090c;
  param_2[8] = DAT_00010910;
  param_2[9] = -0x10000000;
  return;
}



/* ======================================================================
 * 000105b6  txp_build_cts_desc
 * ====================================================================== */

void txp_build_cts_desc(int param_1,int *param_2)

{
  *param_2 = (uint)*(byte *)(DAT_000108f0 + 0x18) * 0x2000 + 0x59000000;
  param_2[1] = DAT_000108f4;
  param_2[2] = DAT_000108f8;
  param_2[3] = DAT_000108fc + -0x10;
  param_2[4] = 0x47000000;
  param_2[5] = (DAT_00010900 + param_1 + DAT_00010904 & 0x7fffffU) + 0x20800000;
  param_2[6] = DAT_00010908 + -4;
  param_2[7] = DAT_0001090c;
  param_2[8] = DAT_00010910;
  param_2[9] = -0x10000000;
  return;
}



/* ======================================================================
 * 00010604  txp_slot_to_irq_bit
 * ====================================================================== */

int txp_slot_to_irq_bit(int param_1)

{
  if (param_1 == 0x1d) {
    return 0x10;
  }
  if (param_1 < 0x1e) {
    if ((param_1 != 0) && (param_1 != 1)) {
      if (param_1 != 0x1b) {
        return DAT_00010914;
      }
      param_1 = 8;
    }
    return param_1;
  }
  if (param_1 == 0x1e) {
    return 0x11;
  }
  if (param_1 != 0x1f) {
    return DAT_00010914;
  }
  return 0x18;
}



/* ======================================================================
 * 00010632  txp_slot_to_bit
 * ====================================================================== */

int txp_slot_to_bit(int param_1)

{
  if (param_1 == 0x1c) {
    return 0x1f;
  }
  if (param_1 == 0x1b) {
    return 0x1e;
  }
  return param_1 + -2;
}



/* ======================================================================
 * 000106aa  txp_set_pipe_enable
 * ====================================================================== */

void txp_set_pipe_enable(uint param_1,int param_2)

{
  uint *puVar1;
  undefined4 uVar2;
  uint uVar3;
  int iVar4;
  ulonglong uVar5;
  
  puVar1 = DAT_00010918;
  if (param_2 == 0) {
    uVar2 = 2;
  }
  else {
    uVar2 = 3;
  }
  uVar3 = DAT_00010918[1];
  if (param_1 - 2 < 0x1b) {
    uVar5 = txp_slot_to_bit(param_1);
    iVar4 = (int)(uVar5 >> 0x20);
    if ((uVar5 & 0x100000000) == 0) {
      if (iVar4 << 0x1e < 0) {
        puVar1[1] = uVar3 & ~(1 << ((uint)uVar5 & 0xff));
        return;
      }
    }
    else if (iVar4 << 0x1e < 0) {
      puVar1[1] = 1 << ((uint)uVar5 & 0xff) | uVar3;
    }
    return;
  }
  iVar4 = DAT_0001091c;
  uVar5 = txp_slot_to_irq_bit(param_1,uVar2);
  uVar3 = (uint)uVar5;
  if ((uVar5 & 0x100000000) == 0) {
    if (param_1 < 0x1d) {
      uVar3 = *(uint *)(iVar4 + 0x30) & ~(1 << (uVar3 & 0xff));
      *(uint *)(iVar4 + 0x30) = uVar3;
      *puVar1 = uVar3;
      return;
    }
    uVar3 = uVar3 + 4;
  }
  uVar3 = *(uint *)(iVar4 + 0x30) | 1 << (uVar3 & 0xff);
  *(uint *)(iVar4 + 0x30) = uVar3;
  *puVar1 = uVar3;
  return;
}



/* ======================================================================
 * 000106b6  txp_program_pipe_slot_ex
 * ====================================================================== */

undefined8 txp_program_pipe_slot_ex(uint param_1,undefined4 param_2,int param_3,int param_4)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  uint uVar4;
  uint uVar5;
  undefined8 uVar6;
  
  iVar3 = 0;
  if (param_1 - 2 < 0x1b) {
    uVar5 = param_1;
    uVar6 = txp_slot_to_bit(param_1);
    iVar1 = DAT_00010918;
    uVar2 = (uint)uVar6;
    if ((int)((ulonglong)uVar6 >> 0x20) == 0) {
      uVar4 = 0x1f;
      if (uVar2 != 0x1e) {
        uVar4 = uVar2;
      }
      *(uint *)(DAT_00010918 + 8) = *(uint *)(DAT_00010918 + 8) & ~(1 << (uVar4 & 0xff));
    }
    else {
      uVar4 = 0x1f;
      if (uVar2 != 0x1e) {
        uVar4 = uVar2;
      }
      *(uint *)(DAT_00010918 + 8) = 1 << (uVar4 & 0xff) | *(uint *)(DAT_00010918 + 8);
    }
    if (param_3 == 0) {
      uVar4 = 0x1f;
      if (uVar2 != 0x1e) {
        uVar4 = uVar2;
      }
      uVar4 = *(uint *)(iVar1 + 0xc) & ~(1 << (uVar4 & 0xff));
    }
    else {
      uVar4 = 0x1f;
      if (uVar2 != 0x1e) {
        uVar4 = uVar2;
      }
      uVar4 = *(uint *)(iVar1 + 0xc) | 1 << (uVar4 & 0xff);
    }
    *(uint *)(iVar1 + 0xc) = uVar4;
    if (param_4 == 0) {
      uVar4 = *(uint *)(iVar1 + 0x10) & ~(1 << (uVar2 & 0xff));
    }
    else {
      uVar4 = 1 << (uVar2 & 0xff) | *(uint *)(iVar1 + 0x10);
    }
    *(uint *)(iVar1 + 0x10) = uVar4 & 0xffffff;
    if (0x11 < uVar5) {
      *(uint *)(iVar1 + 0x18) = *(uint *)(iVar1 + 0x18) & iVar3 << ((uVar2 & 0x7f) << 1);
      return CONCAT44(param_2,param_1);
    }
    *(uint *)(iVar1 + 0x14) = *(uint *)(iVar1 + 0x14) & iVar3 << ((uVar2 & 0x7f) << 1);
  }
  return CONCAT44(param_2,param_1);
}



/* ======================================================================
 * 00010758  txp_build_ba_desc
 * ====================================================================== */

void txp_build_ba_desc(int param_1,int *param_2,uint param_3)

{
  *param_2 = (uint)*(byte *)(DAT_000108f0 + 0x18) * 0x2000 + 0x59000000;
  param_2[1] = DAT_000108f4;
  param_2[2] = DAT_000108f8 + 0x12;
  param_2[3] = DAT_000108fc + -0x40;
  param_2[4] = 0x47000000;
  param_2[5] = (DAT_00010900 + param_1 + DAT_00010904 & 0x7fffffU) + 0x20800000;
  param_2[6] = 0x32000000;
  param_2[7] = (param_3 & 7) + 0x69000000;
  param_2[8] = DAT_0001090c + -8;
  param_2[9] = (param_3 & 7) + 0x68000000;
  param_2[10] = param_3 | 0x60000000;
  param_2[0xb] = DAT_00010910;
  param_2[0xc] = -0x10000000;
  return;
}



/* ======================================================================
 * 000107c8  txp_build_null_desc
 * ====================================================================== */

void txp_build_null_desc(uint *param_1)

{
  *param_1 = *(int *)(DAT_00010920 + 0x30) << 0x10 | 0x20000000;
  *(undefined2 *)(param_1 + 1) = 0;
  *(undefined2 *)((int)param_1 + 6) = 0;
  param_1[2] = 0x58000000;
  return;
}



/* ======================================================================
 * 000107e4  txp_build_resp_descs
 * ====================================================================== */

void txp_build_resp_descs(int param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  uint uVar2;
  int iVar3;
  
  uVar1 = DAT_00010924;
  txp_build_null_desc();
  uVar2 = DAT_00010924 + 0x54;
  txp_build_null_desc();
  if (param_1 == 1) {
    iVar3 = 0x94;
  }
  else if ((param_1 == 2) || (param_1 == 3)) {
    iVar3 = 0x1c;
  }
  else if (param_1 == 0xff) {
    iVar3 = param_2 * 0xc + 0x12;
  }
  else {
    iVar3 = 0;
  }
  txp_build_ba_desc(0,DAT_00010924 + 0xc,0,iVar3,param_4);
  txp_build_ba_desc(1,DAT_00010924 + 0x60,0,iVar3);
  txp_program_pipe_slot(uVar1,2,1,1,0);
  txp_program_pipe_slot(uVar1,0xb,1,1,0);
  txp_program_pipe_slot(uVar2,3,1,1,0);
  txp_program_pipe_slot(uVar2,0xc,1,1,0);
  iVar3 = DAT_00010928;
  *(uint *)(DAT_00010928 + 8) = uVar1 & 0xf6ffffff;
  *(uint *)(iVar3 + 0xc) = uVar2 & 0xf6ffffff;
  *(uint *)(iVar3 + 0x2c) = uVar1 & 0xf6ffffff;
  *(uint *)(iVar3 + 0x30) = uVar2 & 0xf6ffffff;
  return;
}



/* ======================================================================
 * 0001088e  txp_install_resp_descs
 * ====================================================================== */

void txp_install_resp_descs(undefined4 param_1,undefined4 param_2)

{
  txp_program_pipe_slot_ex(2,1,0,0);
  txp_program_pipe_slot_ex(3,1,0,0);
  txp_build_resp_descs(param_1,param_2);
  txp_set_pipe_enable(2,1);
  txp_set_pipe_enable(3,1);
  txp_program_pipe_slot_ex(0xb,1,0,0);
  txp_program_pipe_slot_ex(0xc,1,0,0);
  txp_set_pipe_enable(0xb,1);
  txp_set_pipe_enable(0xc,1);
  return;
}



/* ======================================================================
 * 0001092c  mac_program_own_mac_hw
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void mac_program_own_mac_hw(void)

{
  undefined4 *puVar1;
  int iVar2;
  
  iVar2 = DAT_00010c60;
  puVar1 = DAT_00010c5c;
  _DAT_09c00030 = *DAT_00010c5c;
  _DAT_09c00034 = (uint)*(ushort *)(DAT_00010c5c + 1);
  _DAT_09c00038 = 0x101;
  *(undefined4 *)(DAT_00010c60 + 8) = *(undefined4 *)((int)DAT_00010c5c + 6);
  *(uint *)(iVar2 + 0xc) = (uint)*(ushort *)((int)puVar1 + 10);
  *(undefined4 *)(iVar2 + 0x10) = 0x101;
  return;
}



/* ======================================================================
 * 0001095e  mac_set_own_mac_addr
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void mac_set_own_mac_addr(undefined2 *param_1)

{
  undefined4 *puVar1;
  int iVar2;
  
  puVar1 = DAT_00010c5c;
  *(undefined2 *)DAT_00010c5c = *param_1;
  *(undefined2 *)((int)puVar1 + 2) = param_1[1];
  *(undefined2 *)(puVar1 + 1) = param_1[2];
  *(undefined2 *)((int)puVar1 + 6) = *param_1;
  *(undefined2 *)(puVar1 + 2) = param_1[1];
  *(undefined2 *)((int)puVar1 + 10) = param_1[2];
  *(char *)((int)puVar1 + 0xb) = *(char *)((int)puVar1 + 0xb) + '\x01';
  iVar2 = DAT_00010c60;
  puVar1 = DAT_00010c5c;
  _DAT_09c00030 = *DAT_00010c5c;
  _DAT_09c00034 = (uint)*(ushort *)(DAT_00010c5c + 1);
  _DAT_09c00038 = 0x101;
  *(undefined4 *)(DAT_00010c60 + 8) = *(undefined4 *)((int)DAT_00010c5c + 6);
  *(uint *)(iVar2 + 0xc) = (uint)*(ushort *)((int)puVar1 + 10);
  *(undefined4 *)(iVar2 + 0x10) = 0x101;
  return;
}



/* ======================================================================
 * 00010982  mac_program_timing_regs
 * ====================================================================== */

void mac_program_timing_regs(void)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  iVar1 = DAT_00010c64;
  *(undefined4 *)(DAT_00010c64 + 0x10) = 0x19000000;
  *(undefined4 *)(iVar1 + 0x14) = 0x1c000000;
  *(int *)(iVar1 + 0x18) = iVar1 << 0x13;
  *(int *)(iVar1 + 0x1c) = iVar1 << 0x15;
  *(int *)(iVar1 + 0x20) = iVar1 << 0x16;
  *(int *)(iVar1 + 0x28) = DAT_00010c68;
  *(undefined4 *)(DAT_00010c64 + 0x98) = 0;
  *(undefined4 *)(iVar1 + 0x2c) = DAT_00010c6c;
  *(undefined4 *)(iVar1 + 0x30) = DAT_00010c70;
  *(undefined4 *)(iVar1 + 0x34) = DAT_00010c74;
  *(undefined4 *)(iVar1 + 0x38) = DAT_00010c78;
  *(int *)(iVar1 + 0x3c) = DAT_00010c7c;
  iVar1 = DAT_00010c64;
  *(int *)(DAT_00010c64 + 0x40) = DAT_00010c68 + 0x1e;
  *(int *)(iVar1 + 0x44) = DAT_00010c7c + 0x10;
  iVar2 = DAT_00010c80;
  *(int *)(iVar1 + 0x48) = DAT_00010c80;
  *(int *)(iVar1 + 0x4c) = iVar2 >> 0xe;
  *(undefined4 *)(iVar1 + 0x50) = DAT_00010c84;
  iVar2 = DAT_00010c64;
  *(undefined4 *)(DAT_00010c64 + 0xc0) = 0;
  iVar3 = DAT_00010c68 + -1;
  *(int *)(iVar1 + 0x54) = iVar3;
  iVar3 = iVar3 >> 5;
  *(int *)(iVar1 + 0x5c) = iVar3;
  *(int *)(iVar1 + 0x60) = iVar3 << 2;
  *(undefined4 *)(iVar1 + 100) = DAT_00010c88;
  *(undefined4 *)(iVar1 + 0x68) = 8;
  *(undefined4 *)(iVar1 + 0x6c) = 0;
  iVar3 = DAT_00010c8c;
  *(undefined4 *)(DAT_00010c8c + 0x14) = 0;
  *(undefined4 *)(iVar3 + 0x18) = 0x8000;
  *(undefined4 *)(iVar1 + 0x74) = DAT_00010c90;
  *(undefined4 *)(iVar2 + 0xe4) = 0;
  *(undefined4 *)(iVar1 + 0x78) = DAT_00010c94;
  *(undefined4 *)(iVar2 + 0xe8) = 0x80;
  *(undefined4 *)(iVar2 + 0xf0) = DAT_00010c98;
  *(undefined4 *)(iVar2 + 0xf4) = DAT_00010c9c;
  *(undefined **)(iVar2 + 0xfc) = &DAT_00670000;
  *(undefined4 *)(iVar2 + 0xf8) = 0x60000;
  *(undefined4 *)(DAT_00010ca0 + 8) = 0x70;
  return;
}



/* ======================================================================
 * 00010a2c  mac_program_base_regs
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void mac_program_base_regs(void)

{
  int iVar1;
  uint uVar2;
  int *piVar3;
  
  iVar1 = DAT_00010c60;
  _DAT_09c00000 = DAT_00010ca4;
  _DAT_09c00004 = DAT_00010ca4 + -0x10;
  _DAT_09c00008 = DAT_00010ca4 + -0x20;
  _DAT_09c0000c = DAT_00010ca8;
  _DAT_09c00010 = DAT_00010ca4 + -0x50;
  _DAT_09c00014 = DAT_00010ca4 + -0x40;
  _DAT_09c00018 = DAT_00010cac;
  _DAT_09c0001c = DAT_00010cb0;
  _DAT_09c00020 = DAT_00010cb4;
  _DAT_09c00024 = DAT_00010cb4 << 1;
  _DAT_09c00028 = 0x4000;
  _DAT_09c0002c = DAT_00010cb8;
  *(undefined4 *)(DAT_00010c60 + 0x60) = DAT_00010cbc;
  *(undefined4 *)(iVar1 + 100) = DAT_00010cc0;
  *(undefined4 *)(iVar1 + 0x68) = DAT_00010cc4;
  *(undefined4 *)(iVar1 + 0x6c) = DAT_00010cc8;
  *(int *)(iVar1 + 0x70) = DAT_00010cac + -0xc;
  *(undefined4 *)(iVar1 + 0x74) = DAT_00010ccc;
  *(undefined4 *)(iVar1 + 0x78) = DAT_00010cd0;
  *(undefined4 *)(iVar1 + 0x7c) = DAT_00010cd4;
  iVar1 = DAT_00010c60;
  *(undefined4 *)(DAT_00010c60 + 0x80) = DAT_00010cd8;
  *(undefined4 *)(iVar1 + 0x84) = DAT_00010cdc;
  *(undefined4 *)(iVar1 + 0x88) = DAT_00010ce0;
  *(undefined4 *)(iVar1 + 0x8c) = DAT_00010ce4;
  *(undefined4 *)(iVar1 + 0xb4) = DAT_00010ce8;
  *(undefined4 *)(iVar1 + 0xb8) = 0xf00;
  *(undefined4 *)(iVar1 + 0xbc) = DAT_00010cec;
  mac_program_own_mac_hw();
  *(undefined4 *)(DAT_00010c64 + 0x7c) = 0x2000;
  *(undefined4 *)(DAT_00010ca0 + -0x14) = 4;
  *(undefined4 *)(iVar1 + 0x90) = DAT_00010cf0;
  uVar2 = DAT_00010cf4;
  *(uint *)(iVar1 + 0x94) = DAT_00010cf4;
  iVar1 = DAT_00010c60;
  piVar3 = (int *)(DAT_00010c60 + 0xc0);
  *piVar3 = uVar2 << 0x1f;
  *(uint *)(iVar1 + 0xc4) = (uVar2 & 1) << 0x11;
  *(undefined **)(iVar1 + 200) = &DAT_00160000;
  *(undefined4 *)(iVar1 + 0xcc) = DAT_00010cf8;
  *(undefined4 *)(iVar1 + 0xd0) = DAT_00010cfc;
  *(undefined4 *)(iVar1 + 0xd4) = DAT_00010d00;
  *(int *)(iVar1 + 0xe0) = (int)piVar3 * 0x400;
  *(undefined4 *)(iVar1 + 0xe8) = DAT_00010d04;
  *(int *)(iVar1 + 0xf0) = (int)piVar3 * 0x1000;
  return;
}



/* ======================================================================
 * 00010b1a  mac_program_bssid
 * ====================================================================== */

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void mac_program_bssid(undefined4 *param_1,int param_2)

{
  uint *puVar1;
  uint uVar2;
  
  puVar1 = DAT_00010c60;
  _DAT_09c0003c = *param_1;
  *DAT_00010c60 = (uint)*(ushort *)(param_1 + 1);
  uVar2 = DAT_00010d0c;
  if (param_2 == 1) {
    uVar2 = 0x101;
  }
  puVar1[1] = uVar2;
  return;
}



/* ======================================================================
 * 00010b3e  mac_program_mode_regs
 * ====================================================================== */

void mac_program_mode_regs(void)

{
  int *piVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  
  iVar3 = DAT_00010d14;
  *(undefined4 *)(DAT_00010d14 + 0x34) = DAT_00010d10;
  iVar4 = *(int *)(iVar3 + 100);
  if (iVar4 * 0x2000 < 0) {
    *(undefined4 *)(iVar3 + 0x34) = DAT_00010d18;
  }
  iVar2 = DAT_00010c8c;
  *(undefined4 *)(DAT_00010c8c + 4) = *(undefined4 *)(iVar3 + 0x34);
  *(undefined4 *)(iVar2 + 0x1c) = DAT_00010d1c;
  piVar1 = DAT_00010c64;
  DAT_00010c64[1] = DAT_00010d20;
  if (iVar4 << 0xd < 0) {
    piVar1[1] = piVar1[1] | 0x40000;
  }
  *piVar1 = iVar4;
  *(undefined4 *)(DAT_00010ca0 + 0x10) = 0x78000000;
  return;
}



/* ======================================================================
 * 00010b80  mac_program_mode_sta
 * ====================================================================== */

void mac_program_mode_sta(void)

{
  undefined4 *puVar1;
  int iVar2;
  int iVar3;
  
  mac_program_own_mac_hw();
  iVar3 = DAT_00010ce4 + -0x82;
  *(int *)(DAT_00010d14 + 0x34) = iVar3;
  iVar2 = DAT_00010c8c;
  *(int *)(DAT_00010c8c + 4) = iVar3;
  *(undefined4 *)(iVar2 + 0x1c) = DAT_00010d1c;
  puVar1 = DAT_00010c64;
  DAT_00010c64[1] = &DAT_00198000;
  *puVar1 = *(undefined4 *)(DAT_00010d14 + 100);
  *(undefined4 *)(DAT_00010ca0 + 0x10) = 0x78000000;
  return;
}



/* ======================================================================
 * 00010bb0  mac_program_mode_ap
 * ====================================================================== */

void mac_program_mode_ap(void)

{
  int iVar1;
  undefined4 *puVar2;
  undefined4 *puVar3;
  undefined4 uVar4;
  
  puVar2 = DAT_00010c64;
  DAT_00010c64[1] = &DAT_0004c000;
  uVar4 = DAT_00010d24;
  *puVar2 = DAT_00010d24;
  *(undefined4 *)(DAT_00010d14 + 100) = *puVar2;
  puVar2 = DAT_00010c64;
  DAT_00010c64[0x12] = 0;
  puVar3 = DAT_00010c64;
  DAT_00010c64[0x2e] = 0x40;
  puVar2[0x13] = 0xc00;
  puVar3[0x2f] = 0;
  puVar2[0x16] = 0xc400;
  *(undefined4 *)(DAT_00010ca0 + -0x38) = 0;
  iVar1 = DAT_00010c60;
  *(undefined4 *)(DAT_00010c60 + 0x68) = DAT_00010d28;
  *(undefined4 *)(iVar1 + 0x6c) = DAT_00010cbc;
  *(undefined4 *)(DAT_00010c8c + 0x1c) = uVar4;
  return;
}



/* ======================================================================
 * 00010bfc  mac_noop
 * ====================================================================== */

void mac_noop(void)

{
  return;
}



/* ======================================================================
 * 00010bfe  p2p_set_alt_mac_addr
 * ====================================================================== */

void p2p_set_alt_mac_addr(int param_1,int param_2)

{
  int iVar1;
  int iVar2;
  
  iVar1 = param_2 * 0x3b0 + DAT_00010d2c;
  *(undefined1 *)(iVar1 + 0x3c6) = 0;
  iVar2 = DAT_00010d30;
  if (param_1 != 0) {
    *(undefined1 *)(iVar1 + 0x3c6) = 1;
    *(undefined1 *)(iVar2 + param_2 + 0x46c) = 1;
    iVar2 = param_2 * 6 + iVar2;
    *(undefined2 *)(iVar2 + 0x460) = *(undefined2 *)(iVar2 + 0x454);
    *(undefined2 *)(iVar2 + 0x462) = *(undefined2 *)(iVar2 + 0x456);
    *(undefined2 *)(iVar2 + 0x464) = *(undefined2 *)(iVar2 + 0x458);
    *(byte *)(iVar2 + 0x458) = *(byte *)(iVar2 + 0x458) ^ 0x80;
    mac_program_own_mac_hw();
    mac_program_bssid((undefined2 *)(iVar2 + 0x460),1);
  }
  return;
}



/* ======================================================================
 * 00010d94  wsm_h_0B_join
 * ====================================================================== */

void wsm_h_0B_join(undefined2 *param_1)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = DAT_00011030;
  *(undefined2 **)(DAT_00011030 + -4) = param_1;
  *(undefined1 *)(iVar1 + -9) = *(undefined1 *)(DAT_00011034 + 10);
  iVar1 = wsm_h_0b_join_impl(param_1 + 2);
  if (iVar1 != 0) {
    *param_1 = 0x10;
    uVar2 = wsm_status_from_internal();
    *(undefined4 *)(param_1 + 2) = uVar2;
    *(undefined4 *)(param_1 + 4) = 0;
    *(undefined4 *)(param_1 + 6) = 0;
    hif_send_msg_to_host(param_1);
  }
  return;
}



/* ======================================================================
 * 00010dca  wsm_h_0A_reset
 * ====================================================================== */

void wsm_h_0A_reset(int param_1)

{
  *DAT_00011030 = param_1;
  lmc_post_event_200(*(undefined4 *)(param_1 + 4));
  return;
}



/* ======================================================================
 * 00010dd8  wsm_h_10_set_pm
 * ====================================================================== */

void wsm_h_10_set_pm(undefined2 *param_1)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = DAT_00011038;
  *(undefined1 *)(DAT_00011038 + 0x11) = 1;
  wsm_h_10_set_pm_impl(param_1 + 2);
  uVar2 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar2;
  *param_1 = 8;
  hif_send_msg_to_host(param_1);
  *(undefined1 *)(iVar1 + 0x11) = 0;
  if (*(char *)(iVar1 + 0x12) != '\0') {
    *(undefined1 *)(iVar1 + 0x12) = 0;
    ps_send_pm_complete_ind((uint)*(byte *)(DAT_00011034 + 10) * 0x104 + DAT_0001103c + 0x40);
  }
  return;
}



/* ======================================================================
 * 00010e1a  wsm_h_11_set_bss_params
 * ====================================================================== */

void wsm_h_11_set_bss_params(undefined2 *param_1)

{
  wsm_h_join_apply(param_1 + 2);
  *param_1 = 8;
  *(undefined4 *)(param_1 + 2) = 0;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010e34  wsm_h_0E_start_measure
 * ====================================================================== */

void wsm_h_0E_start_measure(undefined2 *param_1)

{
  undefined4 uVar1;
  
  measure_start();
  *param_1 = 8;
  uVar1 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010e4e  wsm_h_0F_unknown
 * ====================================================================== */

void wsm_h_0F_unknown(undefined2 *param_1)

{
  undefined4 uVar1;
  
  wsm_h_0F_set_state_signal();
  *param_1 = 8;
  uVar1 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010e68  wsm_h_05_read_mib
 * ====================================================================== */

void wsm_h_05_read_mib(short *param_1)

{
  undefined4 uVar1;
  
  func_0xfff0087a(param_1 + 2);
  uVar1 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar1;
  *param_1 = param_1[5] + 0xc;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010e86  wsm_h_06_write_mib
 * ====================================================================== */

void wsm_h_06_write_mib(undefined2 *param_1)

{
  undefined4 uVar1;
  
  func_0xfff000ec(param_1 + 2);
  uVar1 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar1;
  *param_1 = 8;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010f12  wsm_h_17_start
 * ====================================================================== */

void wsm_h_17_start(undefined2 *param_1)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = DAT_00011030;
  *(undefined2 **)(DAT_00011030 + -8) = param_1;
  *(undefined1 *)(iVar1 + -10) = *(undefined1 *)(DAT_00011034 + 10);
  iVar1 = syn_start_wsm_start(param_1 + 2);
  if (iVar1 != 0x16) {
    *param_1 = 8;
    uVar2 = wsm_status_from_internal();
    *(undefined4 *)(param_1 + 2) = uVar2;
    hif_send_msg_to_host(param_1);
  }
  return;
}



/* ======================================================================
 * 00010f42  wsm_h_18_beacon_transmit
 * ====================================================================== */

void wsm_h_18_beacon_transmit(undefined2 *param_1)

{
  undefined4 uVar1;
  
  wsm_h_beacon_transmit_set(param_1 + 2);
  *param_1 = 8;
  uVar1 = wsm_status_from_internal();
  *(undefined4 *)(param_1 + 2) = uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010f5e  wsm_h_1B_update_ie
 * ====================================================================== */

void wsm_h_1B_update_ie(undefined2 *param_1)

{
  undefined4 uVar1;
  
  uVar1 = wsm_h_1B_update_ie_impl(param_1 + 2);
  *param_1 = 8;
  *(undefined4 *)(param_1 + 2) = uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010f76  wsm_h_1D_unknown
 * ====================================================================== */

void wsm_h_1D_unknown(short *param_1)

{
  undefined4 uVar1;
  
  uVar1 = wsm_h_1D_stub_returns_zero();
  *(undefined4 *)(param_1 + 2) = uVar1;
  param_1[1] = (short)DAT_00011040;
  *param_1 = (short)*(undefined4 *)(param_1 + 4) * 0xc + 0xc;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00010f96  wsm_h_20_unknown
 * ====================================================================== */

void wsm_h_20_unknown(undefined2 *param_1)

{
  undefined4 uVar1;
  
  uVar1 = wsm_h_20_vif_param_poke();
  *(undefined4 *)(param_1 + 2) = uVar1;
  param_1[1] = (ushort)*(byte *)(DAT_00011034 + 10) << 6 | 0x420;
  *param_1 = 8;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00011090  FUN_00011090
 * ====================================================================== */

uint FUN_00011090(ushort *param_1,ushort *param_2,uint param_3,ushort *param_4)

{
  ushort uVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  iVar3 = 0;
  iVar4 = 0x10;
  do {
    uVar1 = *param_1;
    iVar4 = (iVar4 + -2) * 0x10000 >> 0x10;
    param_1 = param_1 + 1;
    iVar3 = (uint)uVar1 + iVar3;
  } while (0 < iVar4);
  iVar4 = 0x10;
  do {
    uVar1 = *param_2;
    iVar4 = (iVar4 + -2) * 0x10000 >> 0x10;
    param_2 = param_2 + 1;
    iVar3 = (uint)uVar1 + iVar3;
  } while (0 < iVar4);
  uVar2 = (int)&DAT_00003a00 + (uint)*param_4 + iVar3 + (param_3 * 0x100 + (param_3 >> 8) & 0xffff);
  param_4 = param_4 + 2;
  for (iVar3 = param_3 - 4; iVar3 = iVar3 * 0x10000 >> 0x10, 1 < iVar3; iVar3 = iVar3 + -2) {
    uVar1 = *param_4;
    param_4 = param_4 + 1;
    uVar2 = uVar1 + uVar2;
  }
  if (0 < iVar3) {
    uVar2 = (byte)*param_4 + uVar2;
  }
  return ~((uVar2 >> 0x10) + uVar2) & 0xffff;
}



/* ======================================================================
 * 000110fc  FUN_000110fc
 * ====================================================================== */

void FUN_000110fc(int param_1,ushort *param_2,void *param_3,ushort *param_4,void *param_5,
                 undefined1 param_6,uint param_7)

{
  int iVar1;
  undefined2 uVar2;
  ushort *puVar3;
  void *dst;
  ushort uVar4;
  int iVar5;
  
  puVar3 = *(ushort **)(param_1 * 0x40 + DAT_000114b0 + 0x3c);
  uVar4 = *puVar3;
  iVar1 = (uint)*(byte *)(DAT_000114b4 + (param_7 >> 1)) + ((int)~((uint)uVar4 << 0x18) >> 0x1f) * 2
  ;
  iVar5 = param_1 * 0x98 + DAT_000114bc;
  if (*(int *)(param_1 * 0x3b0 + DAT_000114b8 + 0x1c) * 0x20000000 < 0) {
    *puVar3 = uVar4 & 0xfeff | 0x200;
    puVar3[2] = *param_4;
    puVar3[3] = param_4[1];
    puVar3[4] = param_4[2];
    puVar3[5] = *(ushort *)(iVar5 + 0x482);
    puVar3[6] = *(ushort *)(iVar5 + 0x484);
    puVar3[7] = *(ushort *)(iVar5 + 0x486);
    puVar3[8] = *param_2;
    puVar3[9] = param_2[1];
    uVar4 = param_2[2];
  }
  else {
    *puVar3 = uVar4 & 0xfdff | 0x100;
    puVar3[2] = *(ushort *)(iVar5 + 0x482);
    puVar3[3] = *(ushort *)(iVar5 + 0x484);
    puVar3[4] = *(ushort *)(iVar5 + 0x486);
    puVar3[5] = *param_2;
    puVar3[6] = param_2[1];
    puVar3[7] = param_2[2];
    puVar3[8] = *param_4;
    puVar3[9] = param_4[1];
    uVar4 = param_4[2];
  }
  puVar3[10] = uVar4;
  fw_memcpy((void *)((int)puVar3 + iVar1 + 0x3a),param_5,0x10);
  dst = (void *)((int)puVar3 + iVar1 + 0x2a);
  fw_memcpy(dst,param_3,0x10);
  *(undefined1 *)((int)puVar3 + iVar1 + 0x4e) = param_6;
  fw_memcpy((void *)((int)puVar3 + iVar1 + 0x52),dst,0x10);
  fw_memcpy((void *)((int)puVar3 + iVar1 + 100),param_2,6);
  *(undefined2 *)((int)puVar3 + iVar1 + 0x4c) = 0;
  uVar2 = FUN_00011090(param_3,param_5,
                       CONCAT11(*(undefined1 *)((int)puVar3 + iVar1 + 0x26),
                                *(undefined1 *)((int)puVar3 + iVar1 + 0x27)),
                       (int)puVar3 + iVar1 + 0x4a);
  *(undefined2 *)((int)puVar3 + iVar1 + 0x4c) = uVar2;
  tx_send_template_frame(param_1,7);
  return;
}



/* ======================================================================
 * 00011212  ind_0806_scan_complete
 * ====================================================================== */

void ind_0806_scan_complete(undefined4 *param_1)

{
  undefined2 *puVar1;
  int iVar2;
  undefined8 uVar3;
  
  if ((*(char *)(DAT_000114c0 + 5) == '\0') || (*(int *)(DAT_000114c0 + 8) != 0)) {
    puVar1 = (undefined2 *)hif_alloc_msg_to_host(0xc);
    if (puVar1 == (undefined2 *)0x0) {
      fw_assert(s_wsmlmac_c_000114c8,0x1dd,3);
      return;
    }
    puVar1[1] = (short)DAT_000114c4;
    *puVar1 = 0xc;
    uVar3 = wsm_status_from_internal(*param_1);
    iVar2 = (int)((ulonglong)uVar3 >> 0x20);
    *(int *)(iVar2 + 4) = (int)uVar3;
    *(undefined1 *)(iVar2 + 8) = *(undefined1 *)(param_1 + 1);
    *(undefined1 *)(iVar2 + 9) = *(undefined1 *)((int)param_1 + 5);
    *(undefined2 *)(iVar2 + 10) = *(undefined2 *)((int)param_1 + 6);
    hif_send_msg_to_host(iVar2);
  }
  return;
}



/* ======================================================================
 * 00011260  ind_0809_set_pm_complete
 * ====================================================================== */

void ind_0809_set_pm_complete(short param_1,undefined4 *param_2)

{
  undefined2 *puVar1;
  undefined4 uVar2;
  
  *(undefined1 *)(DAT_000114d4 + 0x1f) = 0;
  puVar1 = (undefined2 *)hif_alloc_msg_to_host(0xc);
  if (puVar1 == (undefined2 *)0x0) {
    fw_assert(s_wsmlmac_c_000114c8,0x204,4);
  }
  else {
    puVar1[1] = param_1 << 6 | (short)DAT_000114c4 + 3U;
    *puVar1 = 0xc;
    *(undefined4 *)(puVar1 + 4) = 0;
    uVar2 = wsm_status_from_internal(*param_2);
    *(undefined4 *)(puVar1 + 2) = uVar2;
    *(undefined1 *)(puVar1 + 4) = *(undefined1 *)(param_2 + 1);
  }
  hif_send_msg_to_host(puVar1);
  return;
}



/* ======================================================================
 * 000112a8  rx_build_indication
 * ====================================================================== */

void rx_build_indication(short *param_1,int param_2,int param_3)

{
  ushort *puVar1;
  int iVar2;
  ushort *src;
  uint n;
  ushort *puVar3;
  
  *param_1 = (short)*(undefined4 *)(param_2 + 0x18) + 0x18;
  param_1[1] = 4;
  iVar2 = DAT_000114c0;
  param_1[2] = 0;
  param_1[3] = 0;
  *(undefined1 *)(param_1 + 4) = *(undefined1 *)(iVar2 + 0xb0);
  *(undefined1 *)((int)param_1 + 9) = 0;
  *(undefined1 *)(param_1 + 5) = 0;
  *(undefined1 *)((int)param_1 + 0xb) = *(undefined1 *)(iVar2 + 0xb1);
  *(int *)(param_1 + 6) = DAT_000114d8 + param_3;
  iVar2 = DAT_000114c0;
  *(undefined4 *)(param_1 + 8) = *(undefined4 *)(DAT_000114c0 + 0xa8);
  *(undefined4 *)(param_1 + 10) = *(undefined4 *)(iVar2 + 0xac);
  puVar1 = (ushort *)(param_1 + 0xc);
  iVar2 = *(int *)(param_2 + 0x18);
  puVar3 = *(ushort **)(param_2 + 0x1c);
  *puVar1 = *puVar3 & 0x42ff | 0x200;
  param_1[0xd] = 0;
  fw_memcpy(param_1 + 0x14,puVar3 + 5,6);
  fw_memcpy(param_1 + 0x11,puVar3 + 2,6);
  fw_memcpy(param_1 + 0xe,puVar3 + 8,6);
  param_1[0x17] = 0;
  src = puVar3 + 0xc;
  n = iVar2 - 0x18U & 0xffff;
  if ((int)((uint)*puVar3 << 0x18) < 0) {
    *puVar1 = *puVar1 & 0xff7f;
    n = n - 2 & 0xffff;
    src = puVar3 + 0xd;
    *param_1 = *param_1 + -2;
  }
  fw_memcpy(param_1 + 0x18,src,n);
  return;
}



/* ======================================================================
 * 00011354  p2p_check_action_frame
 * ====================================================================== */

undefined4
p2p_check_action_frame(int *param_1,undefined4 *param_2,undefined4 param_3,undefined1 *param_4)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = *(int *)(*param_1 + 0x44);
  if ((*(ushort *)(iVar1 + 8) & 0xff) >> 4 == 4) {
    uVar2 = 1;
    if ((*(uint *)(iVar1 + 0x18) & 0xff) >> 4 == 0xe) goto LAB_0001138a;
  }
  else {
    if ((*(byte *)(iVar1 + 8) & 0x7f) >> 5 != 3) {
      return 0;
    }
    uVar2 = 1;
    if (*(char *)(iVar1 + 0x20) == -1) goto LAB_0001138a;
  }
  uVar2 = 0;
LAB_0001138a:
  *param_2 = uVar2;
  *param_4 = 1;
  return 1;
}



/* ======================================================================
 * 00011396  p2p_scan_frames_for_state
 * ====================================================================== */

undefined4
p2p_scan_frames_for_state(undefined4 param_1,undefined1 *param_2,uint param_3,code *param_4)

{
  undefined1 uVar1;
  int iVar2;
  uint uVar3;
  undefined4 uVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  uint local_2c;
  int local_28;
  undefined4 local_24;
  undefined1 *puStack_20;
  uint local_1c;
  code *pcStack_18;
  
  uVar7 = 0;
  local_28 = 0;
  local_2c = 3;
  uVar6 = 3;
  local_24 = param_1;
  puStack_20 = param_2;
  local_1c = param_3;
  pcStack_18 = param_4;
  do {
    if (local_1c <= uVar7) {
      if (uVar6 == 2) {
LAB_000113f0:
        uVar1 = 0;
LAB_000113ce:
        *param_2 = uVar1;
        uVar4 = 1;
      }
      else {
        uVar4 = 0;
      }
      return uVar4;
    }
    iVar2 = (*param_4)(local_24,&local_28,uVar7,&local_2c);
    uVar5 = uVar6;
    if (iVar2 == 1) {
      uVar3 = local_2c & 0xff;
      if (uVar3 != 0) {
        if (local_28 == 1) {
          if ((char)local_2c != '\x02') goto LAB_000113f0;
          uVar1 = 3;
          goto LAB_000113ce;
        }
        uVar5 = uVar3;
        if ((uVar6 != 3) && (uVar5 = uVar6, uVar6 != uVar3)) {
          uVar5 = 0;
        }
      }
    }
    uVar7 = uVar7 + 1;
    uVar6 = uVar5;
  } while( true );
}



/* ======================================================================
 * 000117c8  wsm_arg_validate_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00011836) */
/* WARNING: Removing unreachable block (ram,0x00011836) */

int wsm_arg_validate_dispatch(int param_1,int param_2)

{
  char cVar1;
  byte *pbVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  
  iVar4 = param_1 * 4 + (uint)*(byte *)(param_2 + 0x2a) * 0x3b0 + DAT_000118e0 + 0x18;
  if (*(char *)(iVar4 + 0x251) == '\0') {
    return 0;
  }
  cVar1 = *(char *)(iVar4 + 0x252);
  if (cVar1 == '\x01') {
    if ((*(uint *)(param_2 + 0x10) & 0xfff9ffff) == 0) {
LAB_00011820:
      iVar3 = 1;
      goto LAB_00011826;
    }
  }
  else {
    if (cVar1 == '\x02') {
      iVar3 = *(int *)(param_2 + 0x10) << 0xe;
    }
    else {
      if (cVar1 != '\x03') {
        if (cVar1 == '\x04') goto LAB_00011820;
        goto LAB_00011824;
      }
      iVar3 = *(int *)(param_2 + 0x10) << 0xd;
    }
    if (iVar3 < 0) goto LAB_00011820;
  }
LAB_00011824:
  iVar3 = 0;
LAB_00011826:
  if ((iVar3 == 1) && (uVar5 = (uint)*(byte *)(iVar4 + 0x253), (int)(uVar5 << 0x18) < 0)) {
                    /* WARNING: Could not recover jumptable at 0x00011836. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    if (uVar5 - 0x81 < (uint)DAT_0001183a) {
      pbVar2 = (byte *)(uVar5 + 0x117ba);
    }
    else {
      pbVar2 = (byte *)(DAT_0001183a + 0x1183b);
    }
    iVar4 = (*(code *)((uint)*pbVar2 * 2 + 0x1183b))();
    return iVar4;
  }
  return iVar3;
}



/* ======================================================================
 * 00011992  hif_send_confirm_status8
 * ====================================================================== */

void hif_send_confirm_status8(void)

{
  int iVar1;
  undefined8 uVar2;
  
  *(undefined2 *)*DAT_00011d10 = 8;
  uVar2 = wsm_status_from_internal();
  iVar1 = (int)((ulonglong)uVar2 >> 0x20);
  *(int *)(iVar1 + 4) = (int)uVar2;
  hif_send_msg_to_host(iVar1);
  return;
}



/* ======================================================================
 * 000119aa  hif_send_confirm_status16
 * ====================================================================== */

void hif_send_confirm_status16(undefined4 *param_1)

{
  int iVar1;
  undefined8 uVar2;
  
  **(undefined2 **)(DAT_00011d10 + -4) = 0x10;
  uVar2 = wsm_status_from_internal(*param_1);
  iVar1 = (int)((ulonglong)uVar2 >> 0x20);
  *(int *)(iVar1 + 4) = (int)uVar2;
  *(undefined4 *)(iVar1 + 8) = param_1[1];
  *(undefined4 *)(iVar1 + 0xc) = param_1[2];
  hif_send_msg_to_host(iVar1);
  return;
}



/* ======================================================================
 * 000119d0  ind_080F_join_complete_a
 * ====================================================================== */

void ind_080F_join_complete_a(short param_1,undefined4 param_2)

{
  undefined2 *puVar1;
  int iVar2;
  undefined8 uVar3;
  
  puVar1 = (undefined2 *)hif_alloc_msg_to_host(8);
  iVar2 = DAT_00011d14;
  if (puVar1 != (undefined2 *)0x0) {
    *puVar1 = 8;
    puVar1[1] = param_1 << 6 | (ushort)iVar2;
    uVar3 = wsm_status_from_internal(param_2);
    iVar2 = (int)((ulonglong)uVar3 >> 0x20);
    *(int *)(iVar2 + 4) = (int)uVar3;
    hif_send_msg_to_host(iVar2);
    return;
  }
  fw_assert(DAT_00011d18,DAT_00011d14 - 0x75,0xd);
  return;
}



/* ======================================================================
 * 00011a0a  ind_080A_switch_channel
 * ====================================================================== */

void ind_080A_switch_channel(undefined4 param_1)

{
  undefined2 *puVar1;
  
  puVar1 = (undefined2 *)hif_alloc_msg_to_host(8);
  if (puVar1 != (undefined2 *)0x0) {
    puVar1[1] = (short)DAT_00011d14 + -5;
    *puVar1 = 8;
    *(undefined4 *)(puVar1 + 2) = param_1;
    hif_send_msg_to_host();
    return;
  }
  fw_assert(DAT_00011d18,0x860,6);
  return;
}



/* ======================================================================
 * 00011a38  ind_0808_ba_timeout
 * ====================================================================== */

/* ind_0808_ba_timeout(tid, mac) -- block-ack timeout indication.
   
   *** THIS INDICATION CRASHES THE MAINLINE cw1200 DRIVER. ***
   Verified at instruction level here and against the driver source.
   
   Firmware side (disassembly, not decompile):
     mov  r0,#0xc ; bl hif_alloc_msg_to_host    -> 12-byte message TOTAL
     strb r4,[r0,#0x4]                          -> tid        at msg+4
     strh ...,[r0,#0x6] / #0x8 / #0xa           -> mac[6]     at msg+6..0xB
     ldr  r1,[0x00011d14] ; sub r1,r1,#0x7      -> MsgId = 0x080F - 7 = 0x0808
     strh r1,[r0,#0x2] ; mov r1,#0xc ; strh r1,[r0,#0x0]
   
   So MsgLen = 12 and the payload after the 4-byte header is only **8 bytes**:
       struct { u8 tid; u8 pad; u8 mac[6]; }
   
   Driver side, cw1200 wsm.c wsm_ba_timeout_indication():
       WSM_GET32(buf);            /* consumes payload[0..3] */
       tid = WSM_GET8(buf);       /* payload[4]  */
       WSM_GET8(buf);             /* payload[5]  */
       WSM_GET(buf, addr, 6);     /* payload[6..11]  <-- needs 12 payload bytes */
   
   wsm_buf.end = begin + wsm->len = begin + 12, and data begins at begin+4, so only
   8 payload bytes are available.  The 6-byte MAC read trips
   `if ((buf)->data + size > (buf)->end) goto underflow`, the function returns
   -EINVAL, and cw1200_bh does:
       if (WARN_ON(wsm_handle_rx(...))) goto err;   ->  bh_error = 1
       pr_err("[BH] Fatal error, exiting.\n");
   i.e. a WARN splat and a dead device until the firmware is reloaded.
   
   **Trigger:** any block-ack session timing out.  Reached from
   bab_session_teardown (0x0001A448) via the per-session timer at slot+0x3B4 and
   from bab_teardown_vif_sessions (0x0001A578).  Rare on a healthy link, which is
   why it has not been pinned before -- but it fires exactly when the link is
   already struggling.
   
   **Fix is a one-liner:** delete the leading `WSM_GET32(buf)`.  Then tid lands on
   payload[0], the pad on payload[1] and the MAC on payload[2..7], which matches
   this function byte-for-byte.  The stray u32 is presumably a reserved word in the
   original ST-E API that this firmware build does not emit.
   
   Note mainline's `default:` case for unknown indications is only a pr_warn, so an
   *unhandled* id would have been harmless.  The damage here comes from an id that
   IS handled, with the wrong length. */

void ind_0808_ba_timeout(undefined1 param_1,undefined4 param_2)

{
  undefined2 *puVar1;
  undefined2 unaff_r4;
  undefined2 local_c;
  undefined2 local_a;
  
  puVar1 = (undefined2 *)hif_alloc_msg_to_host(0xc);
  if (puVar1 != (undefined2 *)0x0) {
    *(undefined1 *)(puVar1 + 2) = param_1;
    local_c = (undefined2)param_2;
    puVar1[3] = local_c;
    local_a = (undefined2)((uint)param_2 >> 0x10);
    puVar1[4] = local_a;
    puVar1[5] = unaff_r4;
    puVar1[1] = (short)DAT_00011d14 + -7;
    *puVar1 = 0xc;
    hif_send_msg_to_host();
  }
  return;
}



/* ======================================================================
 * 00011a66  ind_0807_measure_complete
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00011a96) */
/* WARNING: Removing unreachable block (ram,0x00011a96) */
/* ind_0807_measure_complete(ctl) -- 802.11k MEASURE_COMPLETE indication.
   
   *** THIS OVERTURNS A DOCUMENTED CONCLUSION. ***
   WSM-FIRMWARE-MAP.md previously stated: "The verdict: unusable.  There is no
   measurement-complete indication in this firmware."  That is WRONG.  The
   indication exists and its id is **0x0807**:
   
     msg = hif_alloc_msg_to_host(0x48);
     word_copy(msg, ctl, 0x1A);                       /* 104 bytes of results */
     *(u32 *)(msg + 8) = wsm_status_from_internal(ctl->status);
     *(u16 *)(msg + 2) = 0x080F - 8 = 0x0807;         /* MsgId  */
     *(u16 *)(msg + 0) = 0x10;                        /* MsgLen, BASE value */
     switch8 on *(u8 *)(msg + 5)  ->  per-measurement-type handler(0x30, 0x28)
   
   Chain that proves it is the measurement path:
     measure_start (0x00012BC0, WSM 0x000E)  ->  measure_emit_complete (0x00012D88)
     ->  this function  ->  hif_send_msg_to_host
   
   Why the earlier search missed it: that search looked for a call to
   hif_send_msg_to_host from within the measurement module itself.  The emission is
   two levels removed, and 0x00012D88 is shared with six other call sites, so the
   measurement module never references the HIF layer directly.
   
   MsgLen 0x10 is only the BASE; 0x48 bytes are allocated and the switch8 handler
   selected by the measurement type at msg+5 fixes up the final length, so the
   indication is variably sized.  The exact per-type lengths have NOT been
   determined -- do that before writing a parser.
   
   Driver status: mainline cw1200 has no 0x0807 case, so it currently falls to the
   `default: pr_warn` arm.  That is harmless (unlike 0x0808, see
   ind_0808_ba_timeout), so adding support is purely additive. */

void ind_0807_measure_complete(int param_1)

{
  undefined2 *puVar1;
  undefined4 uVar2;
  uint uVar3;
  
  puVar1 = (undefined2 *)hif_alloc_msg_to_host(0x48);
  if (puVar1 != (undefined2 *)0x0) {
    fw_memcpy_words(puVar1,param_1,0x1a);
    uVar2 = wsm_status_from_internal(*(undefined4 *)(param_1 + 8));
    *(undefined4 *)(puVar1 + 4) = uVar2;
    puVar1[1] = (short)DAT_00011d14 + -8;
    *puVar1 = 0x10;
    uVar3 = (uint)*(byte *)((int)puVar1 + 5);
                    /* WARNING: Could not recover jumptable at 0x00011a96. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    if (DAT_00011a9a <= uVar3) {
      uVar3 = (uint)DAT_00011a9a;
    }
    (*(code *)((uint)*(byte *)(uVar3 + 0x11a9b) * 2 + 0x11a9b))(0x30,0x28);
    return;
  }
  fw_assert(DAT_00011d18,DAT_00011d1c,10);
  return;
}



/* ======================================================================
 * 00011ae6  vif_init_action_filter_table
 * ====================================================================== */

void vif_init_action_filter_table(int param_1)

{
  int iVar1;
  
  iVar1 = param_1 * 0x3b0 + DAT_00011d20;
  fw_memzero((void *)(iVar1 + 0x268),0x2c);
  *(undefined4 *)(iVar1 + 0x264) = 3;
  *(undefined1 *)(iVar1 + 0x269) = 3;
  *(undefined1 *)(iVar1 + 0x26a) = 4;
  *(undefined1 *)(iVar1 + 0x26b) = 0x85;
  *(undefined1 *)(iVar1 + 0x26d) = 3;
  *(undefined1 *)(iVar1 + 0x26e) = 4;
  *(undefined1 *)(iVar1 + 0x26f) = 0x89;
  *(undefined1 *)(iVar1 + 0x271) = 3;
  *(undefined1 *)(iVar1 + 0x272) = 4;
  *(undefined1 *)(iVar1 + 0x273) = 0x81;
  *(undefined1 *)(iVar1 + 0x275) = 3;
  *(undefined1 *)(iVar1 + 0x276) = 4;
  *(undefined1 *)(iVar1 + 0x277) = 0x86;
  *(undefined1 *)(iVar1 + 0x279) = 3;
  *(undefined1 *)(iVar1 + 0x27a) = 4;
  *(undefined1 *)(iVar1 + 0x27b) = 0x82;
  *(undefined1 *)(iVar1 + 0x27d) = 3;
  *(undefined1 *)(iVar1 + 0x27e) = 4;
  *(undefined1 *)(iVar1 + 0x27f) = 0x83;
  *(undefined1 *)(iVar1 + 0x281) = 3;
  *(undefined1 *)(iVar1 + 0x282) = 4;
  *(undefined1 *)(iVar1 + 0x283) = 0x87;
  *(undefined1 *)(iVar1 + 0x285) = 3;
  *(undefined1 *)(iVar1 + 0x286) = 4;
  *(undefined1 *)(iVar1 + 0x287) = 0x88;
  *(undefined1 *)(iVar1 + 0x289) = 3;
  *(undefined1 *)(iVar1 + 0x28a) = 4;
  *(undefined1 *)(iVar1 + 0x28b) = 0x84;
  *(undefined1 *)(iVar1 + 0x28d) = 3;
  *(undefined1 *)(iVar1 + 0x28e) = 4;
  *(undefined1 *)(iVar1 + 0x28f) = 0x8a;
  return;
}



/* ======================================================================
 * 00011ba2  wsm_h_20_vif_param_poke
 * ====================================================================== */

undefined4 wsm_h_20_vif_param_poke(int param_1)

{
  uint uVar1;
  int iVar2;
  int iVar3;
  
  if (1 < *(byte *)(DAT_00011d20 + 10)) {
    return 1;
  }
  iVar3 = (uint)*(byte *)(DAT_00011d20 + 10) * 0x3b0 + DAT_00011d20;
  uVar1 = (uint)*(byte *)(param_1 + 4);
  iVar2 = (uint)*(byte *)(DAT_00011d20 + 10) * 0x3b0 + DAT_00011d20;
  if (uVar1 == 0x40) {
    uVar1 = *(uint *)(iVar2 + 0x264) & 0xfffffffe;
  }
  else {
    if (uVar1 != 0x41) {
      if (*(int *)(iVar2 + 0x264) << 0x1e < 0) {
        if (uVar1 == 0x43) {
          fw_memzero((void *)(iVar3 + 0x268),4);
          fw_memzero((void *)(iVar3 + 0x26c),4);
          fw_memzero((void *)(iVar3 + 0x270),4);
          fw_memzero((void *)(iVar3 + 0x274),4);
          fw_memzero((void *)(iVar3 + 0x278),4);
          uVar1 = *(uint *)(iVar2 + 0x264) & 0xfffffffd;
          goto LAB_00011c3c;
        }
      }
      else {
        if (uVar1 == 0x42) {
          vif_init_action_filter_table();
          return 0;
        }
        if (*(byte *)(param_1 + 5) < 0xb) {
          if (*(byte *)(param_1 + 5) == 0) {
            fw_memzero((void *)(uVar1 * 4 + iVar3 + 0x18 + 0x24c),4);
            return 0;
          }
          fw_memcpy((void *)(uVar1 * 4 + iVar3 + 0x18 + 0x24c),(void *)(param_1 + 4),4);
          return 0;
        }
      }
      return 1;
    }
    uVar1 = *(uint *)(iVar2 + 0x264) & 0xfffffffe | 1;
  }
LAB_00011c3c:
  *(uint *)(iVar2 + 0x264) = uVar1;
  return 0;
}



/* ======================================================================
 * 00011c7c  FUN_00011c7c
 * ====================================================================== */

undefined4 FUN_00011c7c(int param_1,int param_2,undefined4 *param_3)

{
  undefined4 uVar1;
  int iVar2;
  char *pcVar3;
  uint uVar4;
  undefined4 local_34;
  undefined4 uStack_30;
  undefined4 uStack_2c;
  undefined4 local_28;
  int local_24;
  int iStack_20;
  int iStack_1c;
  undefined4 *local_18;
  
  local_34 = DAT_00011d24;
  uStack_30 = DAT_00011d28;
  uStack_2c = DAT_00011d2c;
  local_28 = DAT_00011d30;
  uVar4 = (int)(((uint)*(byte *)(param_2 + 5) + (uint)*(byte *)(param_2 + 4) * 0x100) * 0x10000) >>
          0x10;
  if ((*(char *)(param_2 + 6) == ':') && (pcVar3 = (char *)(param_2 + 0x28), *pcVar3 == -0x79)) {
    local_24 = param_2 + 0x18;
    iStack_20 = param_1;
    iStack_1c = param_2;
    local_18 = param_3;
    iVar2 = fw_mem_equal(local_24,0x10,param_1 + 4,0x10);
    if ((iVar2 == 0) && (iVar2 = fw_mem_equal(local_24,0x10,&local_34,0x10), iVar2 == 0)) {
      uStack_2c = CONCAT13(1,(undefined3)uStack_2c);
      local_28 = CONCAT13(*(undefined1 *)(param_1 + 0x13),
                          CONCAT12(*(undefined1 *)(param_1 + 0x12),
                                   CONCAT11(*(undefined1 *)(param_1 + 0x11),0xff)));
      iVar2 = fw_mem_equal(local_24,0x10,&local_34,0x10);
      if (iVar2 == 0) {
        return 3;
      }
    }
    if ((((*(char *)(param_2 + 0x29) == '\0') && (0x17 < (int)uVar4)) &&
        (*(char *)(param_2 + 7) == -1)) &&
       (uVar4 = FUN_00011090(param_2 + 8,local_24,uVar4 & 0xffff,pcVar3),
       *(ushort *)(param_2 + 0x2a) == uVar4)) {
      *local_18 = pcVar3;
      uVar1 = 2;
    }
    else {
      uVar1 = 1;
    }
  }
  else {
    uVar1 = 0;
  }
  return uVar1;
}



/* ======================================================================
 * 00011dac  lmc_req_count_pending
 * ====================================================================== */

uint lmc_req_count_pending
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  uint uVar2;
  uint extraout_r1;
  uint extraout_r1_00;
  uint uVar3;
  
  fw_div_scaled(*(byte *)(DAT_00012158 + 3) + 0x1d,0x1e,param_3,param_4,param_4);
  iVar1 = DAT_0001215c;
  uVar3 = 0;
  while( true ) {
    fw_div_scaled(((extraout_r1 & 0xff) - uVar3) + 0x1e,0x1e);
    uVar2 = (uint)*(byte *)(iVar1 + (extraout_r1_00 & 0xff) + 0x4c04);
    if (-1 < (int)(uVar2 << 0x1e)) {
      return uVar3;
    }
    if ((int)(uVar2 << 0x1d) < 0) break;
    uVar3 = uVar3 + 1 & 0xff;
    if (0x1d < uVar3) {
      return uVar3;
    }
  }
  return uVar3;
}



/* ======================================================================
 * 00011df0  task_wsmlmac_11df0
 * ====================================================================== */

void task_wsmlmac_11df0(void)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  ushort *puVar4;
  uint extraout_r1;
  uint extraout_r1_00;
  uint uVar5;
  uint uVar6;
  
  uVar2 = lmc_req_count_pending();
  iVar1 = DAT_00012158;
  fw_div_scaled((*(byte *)(DAT_00012158 + 3) - uVar2) + 0x1e,0x1e);
  uVar5 = extraout_r1 & 0xff;
  for (uVar6 = 0; iVar3 = DAT_0001215c, uVar6 < uVar2; uVar6 = uVar6 + 1 & 0xff) {
    *(byte *)(DAT_0001215c + uVar5 + 0x4c04) = *(byte *)(DAT_0001215c + uVar5 + 0x4c04) | 4;
    iVar3 = uVar5 * 4 + iVar3;
    puVar4 = (ushort *)(*(int *)(iVar3 + 0x4b8c) + 0x18);
    if ((*(char *)(iVar1 + 0x14) != '\0') &&
       (((int)uVar6 < (int)(uVar2 - 1) || (*(char *)(iVar1 + 0x15) != '\0')))) {
      *puVar4 = *puVar4 | 0x2000;
    }
    wsm_h_04_tx_req(*(undefined4 *)(iVar3 + 0x4b8c));
    __udivsi3(uVar5 + 0x1f,0x1e);
    uVar5 = extraout_r1_00;
  }
  *(undefined1 *)(iVar1 + 0x14) = 0;
  return;
}



/* ======================================================================
 * 00011e6c  chanswitch_complete
 * ====================================================================== */

void chanswitch_complete(void)

{
  *(undefined1 *)(DAT_00012160 + 4) = 0;
  ind_080A_switch_channel();
  return;
}



/* ======================================================================
 * 00011e7a  chanswitch_apply
 * ====================================================================== */

void chanswitch_apply(void)

{
  undefined2 uVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  
  iVar3 = DAT_00012160;
  iVar2 = DAT_0001215c;
  uVar1 = *(undefined2 *)(DAT_00012160 + 10);
  iVar4 = (uint)*(byte *)(DAT_00012160 + 5) * 0x3b0 + DAT_0001215c;
  iVar5 = iVar4 + 0x18;
  syn_start_release_channel_use(iVar5);
  *(undefined2 *)(iVar4 + 0x42) = uVar1;
  *(undefined2 *)(iVar2 + 2) = uVar1;
  mac_set_channel(uVar1);
  syn_start_register_channel_use(iVar5);
  if (*(char *)(iVar3 + 8) == '\x01') {
    *(byte *)(DAT_00012164 + 0x15) = *(byte *)(DAT_00012164 + 0x15) & 0xfd;
    evt_flags_set(DAT_00012168,0x200000);
  }
  *(undefined1 *)(DAT_00012158 + 0x65) = 0;
  chanswitch_complete();
  return;
}



/* ======================================================================
 * 00011ecc  task_11ecc
 * ====================================================================== */

void task_11ecc(void)

{
  int iVar1;
  int iVar2;
  char *pcVar3;
  int iVar4;
  
  iVar1 = DAT_00012160;
  pcVar3 = (char *)(DAT_00012160 + 8);
  *(undefined1 *)(DAT_00012158 + 0x65) = 1;
  if (*pcVar3 == '\x01') {
    *(byte *)(DAT_00012164 + 0x15) = *(byte *)(DAT_00012164 + 0x15) | 2;
  }
  if (*(char *)(iVar1 + 9) != '\0') {
    iVar4 = (uint)*(byte *)(DAT_00012160 + 5) * 0x3b0 + DAT_0001215c;
    tsf_read(*(undefined1 *)(iVar4 + 0x1a));
    iVar2 = *(int *)(DAT_0001216c + 8);
    __udivmoddi4();
    iVar4 = *(int *)(iVar4 + 0x118);
    if (1999 < (int)((*(byte *)(iVar1 + 9) - 1) * iVar4 + (iVar4 - iVar2))) {
      timer_start(DAT_00012158 + 0x48,4000);
      return;
    }
  }
  chanswitch_apply();
  return;
}



/* ======================================================================
 * 00011f3a  lmc_flush_pending_tx
 * ====================================================================== */

void lmc_flush_pending_tx(void)

{
  int iVar1;
  int iVar2;
  int iVar3;
  int *piVar4;
  
  iVar1 = DAT_00012158;
  piVar4 = (int *)(DAT_00012158 + 0x20);
  *(undefined1 *)(DAT_00012158 + 0x2b) = 0;
  iVar2 = *piVar4;
  *piVar4 = 0;
  *(undefined4 *)(iVar1 + 0x24) = 0;
  while (iVar3 = DAT_00012160, iVar2 != 0) {
    iVar3 = *(int *)(iVar2 + 4);
    tx_frame_complete(iVar2,0x14);
    iVar2 = iVar3;
  }
  *(undefined4 *)(DAT_00012160 + -0x68) = 0;
  *(undefined1 *)(iVar3 + 0x14) = 0;
  *(undefined4 *)(iVar3 + 0x1c) = 0;
  *(undefined2 *)(iVar1 + 0x28) = 0;
  return;
}



/* ======================================================================
 * 00011f6c  beacon_pick_soonest_vif
 * ====================================================================== */

undefined8 beacon_pick_soonest_vif(undefined4 param_1,int param_2)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  undefined8 uVar7;
  uint local_18;
  
  uVar3 = 0;
  local_18 = 0;
  iVar4 = DAT_00012170;
  do {
    iVar6 = DAT_0001215c + uVar3 + DAT_00012174;
    if (*(char *)(iVar6 + 0x1e) != '\0') {
      uVar7 = tsf_read(2);
      param_2 = (int)uVar7;
      iVar5 = *(int *)(DAT_0001216c + 8);
      uVar7 = __udivmoddi4(param_2,(int)((ulonglong)uVar7 >> 0x20),iVar5,0,uVar7);
      uVar7 = u64_add_u32((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),1);
      iVar1 = u64_mul_acc_u32((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),iVar5);
      uVar2 = (uint)*(byte *)(iVar6 + 0x1c);
      if (*(byte *)(iVar6 + 0x1e) == uVar2) {
        uVar2 = 0;
      }
      iVar1 = ((uint)*(ushort *)(uVar3 * 2 + DAT_0001215c + DAT_00012174 + 0x24) * 0x400 +
              iVar5 * (uVar2 - 1) + (iVar1 - param_2)) - *DAT_00012178;
      if (iVar1 < 1) {
        iVar1 = iVar5 * (uint)*(byte *)(iVar6 + 0x1e) + iVar1;
      }
      if (iVar1 < iVar4) {
        iVar4 = iVar1;
        local_18 = uVar3;
      }
    }
    uVar3 = uVar3 + 1;
  } while (uVar3 < 2);
  *(char *)(DAT_00012158 + -0xd7) = (char)local_18;
  if (iVar4 == DAT_00012170) {
    iVar4 = 0;
  }
  return CONCAT44(param_2,iVar4);
}



/* ======================================================================
 * 00012000  beacon_arm_multi_vif_timer
 * ====================================================================== */

void beacon_arm_multi_vif_timer(void)

{
  int iVar1;
  int iVar2;
  
  iVar1 = DAT_00012158;
  mac_set_txop_limit(*DAT_00012178 +
                     (uint)*(ushort *)
                            ((uint)*(byte *)(DAT_00012158 + -0xd7) * 2 + DAT_0001215c +
                            DAT_00012174 + 0x20) * 0x400);
  *(undefined1 *)(iVar1 + -0xd8) = 0;
  iVar2 = beacon_pick_soonest_vif();
  if (0 < iVar2) {
    timer_start(DAT_0001217c,iVar2);
    *(undefined1 *)(iVar1 + -0xd8) = 1;
  }
  return;
}



/* ======================================================================
 * 0001203c  mac_radio_stop_wrapper
 * ====================================================================== */

void mac_radio_stop_wrapper(void)

{
  *(undefined2 *)(DAT_00012158 + 0x28) = 0;
  mac_radio_stop();
  return;
}



/* ======================================================================
 * 0001204c  beacon_dtim_countdown
 * ====================================================================== */

void beacon_dtim_countdown(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  char cVar5;
  
  iVar2 = DAT_00012174;
  iVar1 = DAT_0001215c;
  uVar3 = 0;
  do {
    iVar4 = iVar1 + uVar3 + iVar2;
    cVar5 = *(char *)(iVar4 + 0x1c) + -1;
    *(char *)(iVar4 + 0x1c) = cVar5;
    if (cVar5 == '\0') {
      *(undefined1 *)(iVar4 + 0x1c) = *(undefined1 *)(iVar4 + 0x1e);
    }
    uVar3 = uVar3 + 1 & 0xff;
  } while (uVar3 < 2);
  return;
}



/* ======================================================================
 * 00012074  chanswitch_request
 * ====================================================================== */

undefined4 chanswitch_request(void *param_1)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = DAT_00012160;
  uVar2 = 0;
  *(undefined1 *)(DAT_00012160 + 5) = *(undefined1 *)(DAT_0001215c + 10);
  fw_memcpy((void *)(iVar1 + 8),param_1,4);
  if (*(char *)(iVar1 + 4) == '\0') {
    *(undefined1 *)(iVar1 + 4) = 1;
    evt_flags_set(DAT_00012168,0x1000);
  }
  else {
    uVar2 = 0xf;
  }
  return uVar2;
}



/* ======================================================================
 * 000120a6  wsm_h_15_upload_blob_0x68
 * ====================================================================== */

/* WSM command 0x0015 backend -- uploads a 0x68-byte (104) config blob.
   
     memcpy(g_wsm_cmd15_blob, req_payload, 0x68);   /* 0x04008594 */
     return 0;
   
   That is the whole handler: no validation, no side effect, no signal.
   
   WHAT IT CONFIGURES IS NOT ESTABLISHED.  What is known:
   
   * The destination 0x04008594..0x040085FC is in **bss** (bss begins at
     0x04002078), so it is all zeros at boot and stays that way -- no driver
     issues command 0x0015.
   * It sits immediately before g_chanswitch_ctl (0x040085F8), the control
     block that command 0x0016 SWITCH_CHANNEL drives (+4 busy flag, +5 if_id,
     +8 the 4-byte args, woken with event 0x1000), and the 0x0015 handler
     lives in the same translation unit as chanswitch_request (0x00012074).
     So the blob is in the channel/PHY configuration area.
   * Two struct bases fall inside the blob's address range -- 0x040085D8
     (blob+0x44) and 0x040085F8 (blob+0x64) -- but the fields those bases are
     actually used with (+0x30, +8) resolve PAST the blob's end, so they are
     neighbouring structures rather than consumers.  No code was found that
     reads the blob through a base lying within it.
   
   Attempts that did not resolve it: literal-pool correlation over the whole
   image, and following the two in-range bases into phy_recompute_rate_cfg
   (0x0001524E) and FUN_00005028.  Both of those instead read a global rate
   mask at *(u32 *)(0x04008608) which is outside the blob.
   
   Next idea if this matters: set a watchpoint / dump the region on hardware
   after a scan and a channel switch, since a static consumer search has been
   exhausted. */

undefined4 wsm_h_15_upload_blob_0x68(void *param_1)

{
  fw_memcpy((void *)(DAT_00012160 + -100),param_1,0x68);
  return 0;
}



/* ======================================================================
 * 000120ba  vif_stop_beaconing
 * ====================================================================== */

void vif_stop_beaconing(int param_1)

{
  int iVar1;
  uint uVar2;
  
  iVar1 = param_1 * 0x3b0 + DAT_0001215c;
  uVar2 = *(uint *)(iVar1 + 0x1c);
  if ((int)(uVar2 << 0xe) < 0) {
    *(uint *)(iVar1 + 0x1c) = uVar2 & 0xfffdffff | 0x40000;
    beacon_stop_for_vif();
  }
  return;
}



/* ======================================================================
 * 000120e0  wsm_h_beacon_transmit_set
 * ====================================================================== */

undefined4 wsm_h_beacon_transmit_set(char *param_1)

{
  uint uVar1;
  int iVar2;
  uint uVar3;
  
  uVar3 = (uint)*(byte *)(DAT_0001215c + 10);
  iVar2 = uVar3 * 0x3b0 + DAT_0001215c;
  if (-1 < *(int *)(iVar2 + 0x1c) << 0x1b) {
    return 8;
  }
  if (*param_1 == '\x01') {
    if (-1 < *(int *)(iVar2 + 0x1c) << 0xe) {
      vif_stop_beaconing(uVar3);
    }
    syn_start_load_beacon_template(uVar3,0);
    uVar1 = *(uint *)(iVar2 + 0x1c);
    *(uint *)(iVar2 + 0x1c) = uVar1 & 0xfeffffff;
    if ((int)(uVar1 << 0xe) < 0) {
      beacon_fill_tim(uVar3);
    }
    else {
      *(uint *)(iVar2 + 0x1c) = uVar1 & 0xfefbffff | 0x20000;
      beacon_schedule_next(uVar3);
    }
    if (-1 < *(int *)(iVar2 + 0x1c) << 10) {
      ps_resync_beacon_state(uVar3);
    }
  }
  else if (*param_1 == '\0') {
    vif_stop_beaconing(uVar3);
  }
  return 0;
}



/* ======================================================================
 * 00012180  wsm_h_1B_update_ie_impl
 * ====================================================================== */

/* wsm_h_1b_update_ie_impl(req) -- WSM 0x001B UPDATE_IE backend.
   
     req: { u16 flags; u16 count; <IE>... }   /* IEs packed back to back */
   
     for (i = 0; i < count; i++) {
         if (flags & 1)  template_replace_ie(if_id, ie, 1);   /* BEACON         */
         if (flags & 2)  template_replace_ie(if_id, ie, 5);   /* PROBE_RESPONSE */
         if (flags & 4)  template_replace_ie(if_id, ie, 0);   /* PROBE_REQUEST  */
         ie += ie[1] + 2;
     }
   
   So the flags bitmask matches cw1200's WSM_UPDATE_IE_* exactly: bit 0 beacon,
   bit 1 probe response, bit 2 probe request.  Return value is whatever the last
   template_replace_ie returned (0 ok, 4 = would overflow the 700-byte cap, etc.),
   so a caller that pushes too many IEs gets a non-zero status rather than silent
   truncation.
   
   On the beacon path it does two extra things:
   
   * re-reads the TIM element (id 5) from the live beacon and sets the response
     template's rate index to 5 or 7 depending on TIM bit 0 -- i.e. group-addressed
     traffic pending flips the probe-response rate;
   * when the vif is an AP/GO (state 4 or 6) with inactivity monitoring enabled and
     the updated IE *is* the TIM, it walks the 16-entry link map and updates each
     link's "buffered traffic" flag from the TIM bitmap, maintaining the aggregate
     mask at g_lmc+0x16.
   
   That last part matters for AP mode: **the firmware derives per-link PS state from
   whatever TIM the host writes via UPDATE_IE**, so a driver that writes a stale or
   zeroed TIM will silently clear the per-link buffered-traffic flags.  mainline sets
   the TIM through this path via `cw1200_update_beaconing`/`cw1200_upload_beacon`. */

undefined4 wsm_h_1B_update_ie_impl(ushort *param_1)

{
  ushort uVar1;
  int iVar2;
  undefined1 uVar3;
  ushort uVar4;
  int iVar5;
  uint uVar6;
  uint uVar7;
  ushort *puVar8;
  undefined1 local_2c [4];
  uint local_28;
  char *local_24;
  uint local_20;
  undefined4 local_1c;
  int local_18;
  
  iVar2 = DAT_000124e4;
  local_1c = 0;
  local_20 = (uint)param_1[1];
  local_28 = (uint)*(byte *)(DAT_000124e0 + 10);
  puVar8 = param_1 + 2;
  local_24 = (char *)(local_28 * 0x3b0 + DAT_000124e0 + 0x18);
  do {
    uVar7 = local_20 - 1 & 0xffff;
    if (local_20 == 0) {
      return local_1c;
    }
    local_20 = uVar7;
    if ((*param_1 & 1) != 0) {
      uVar7 = 0;
      local_1c = template_replace_ie(local_28,puVar8,1);
      local_18 = local_28 * 0x70 + DAT_000124e8;
      iVar5 = ie_find_in_frame(*(undefined4 *)(local_18 + 0x10),*(undefined2 *)(local_18 + 0x18),5,0
                              );
      if ((*(byte *)(iVar5 + 4) & 1) == 0) {
        uVar3 = 5;
      }
      else {
        uVar3 = 7;
      }
      *(undefined1 *)(local_18 + 0x1d) = uVar3;
      if ((((local_24[0x389] != '\0') && (local_24[0x388] != '\0')) &&
          ((*local_24 == '\x04' || (*local_24 == '\x06')))) && ((char)*puVar8 == '\x05')) {
        uVar1 = *(ushort *)((int)puVar8 + 5);
        for (; uVar7 < *(ushort *)(iVar2 + 0x14); uVar7 = uVar7 + 1 & 0xff) {
          iVar5 = uVar7 * 0xc + DAT_000124e0 + DAT_000124ec;
          uVar6 = (uint)*(byte *)(iVar5 + 0x18);
          if ((uVar6 == 0) || ((1 << uVar6 & (uint)uVar1) == 0)) {
            *(undefined1 *)(iVar5 + 0x1d) = 0;
            if (*(char *)(iVar5 + 0x1c) == '\0') {
              uVar4 = *(ushort *)(iVar2 + 0x16) & ~(ushort)(1 << uVar6);
              goto LAB_0001224e;
            }
          }
          else {
            *(undefined1 *)(iVar5 + 0x1d) = 1;
            uVar4 = *(ushort *)(iVar2 + 0x16) | (ushort)(1 << uVar6);
LAB_0001224e:
            *(ushort *)(iVar2 + 0x16) = uVar4;
          }
        }
      }
      local_2c[0] = 1;
      if (*(int *)(local_24 + 4) << 0xe < 0) {
        wsm_h_beacon_transmit_set(local_2c);
      }
    }
    if ((int)((uint)*param_1 << 0x1e) < 0) {
      local_1c = template_replace_ie(local_28,puVar8,5);
    }
    if ((int)((uint)*param_1 << 0x1d) < 0) {
      local_1c = template_replace_ie(local_28,puVar8,0);
    }
    puVar8 = (ushort *)((int)puVar8 + *(byte *)((int)puVar8 + 1) + 2);
  } while( true );
}



/* ======================================================================
 * 000122b0  dbg_stats_config_apply
 * ====================================================================== */

void dbg_stats_config_apply(byte *param_1)

{
  int iVar1;
  undefined4 *puVar2;
  
  iVar1 = DAT_000124f0;
  *(undefined4 *)(DAT_000124f0 + 0x3c) = 1;
  puVar2 = DAT_000124f4;
  if ((*param_1 & 1) == 0) {
    *(undefined2 *)DAT_000124f4 = *(undefined2 *)param_1;
    *(undefined2 *)((int)puVar2 + 2) = *(undefined2 *)(param_1 + 2);
    *(undefined2 *)(puVar2 + 1) = *(undefined2 *)(param_1 + 4);
    dbg_set_period(*puVar2);
    mac_set_own_mac_addr(DAT_000124f4);
    *(uint *)(iVar1 + 0x3c) = *(uint *)(iVar1 + 0x3c) | 2;
  }
  return;
}



/* ======================================================================
 * 000122e2  measure_store_slot_params
 * ====================================================================== */

void measure_store_slot_params(int param_1,int param_2)

{
  int iVar1;
  undefined2 *puVar2;
  int iVar3;
  
  iVar1 = DAT_000124e0;
  iVar3 = DAT_000124e0 + param_2 + DAT_000124f8;
  *(undefined1 *)(iVar3 + 0x1c) = *(undefined1 *)(param_1 + 2);
  *(undefined1 *)(iVar3 + 0x1e) = *(undefined1 *)(param_1 + 3);
  puVar2 = (undefined2 *)(param_2 * 2 + iVar1 + DAT_000124f8 + 0x20);
  *puVar2 = *(undefined2 *)(param_1 + 4);
  puVar2[2] = *(undefined2 *)(param_1 + 6);
  return;
}



/* ======================================================================
 * 00012308  lmc_req_slot_enqueue
 * ====================================================================== */

void lmc_req_slot_enqueue(int param_1)

{
  int iVar1;
  undefined1 extraout_r1;
  uint extraout_r1_00;
  int iVar2;
  
  iVar1 = DAT_000124fc;
  __udivsi3(*(byte *)(DAT_000124fc + 2) + 1,0x1e);
  if (extraout_r1_00 == *(byte *)(iVar1 + 3)) {
    fw_assert(s_lmc_c_00012504,DAT_00012500,0x31);
  }
  iVar2 = DAT_000124fc + -0x1c;
  if (*(char *)((uint)*(byte *)(iVar1 + 2) + iVar2) != '\0') {
    fw_assert(s_lmc_c_00012504,DAT_00012500 + 5,0x32);
  }
  *(int *)((uint)*(byte *)(iVar1 + 2) * 4 + DAT_000124e0 + 0x4b8c) = param_1;
  *(undefined1 *)((uint)*(byte *)(iVar1 + 2) + iVar2) = 1;
  __udivsi3(*(byte *)(iVar1 + 2) + 1,0x1e);
  *(undefined1 *)(iVar1 + 2) = extraout_r1;
  if ((*(byte *)(param_1 + 4) & 1) != 0) {
    *(ushort *)(iVar1 + 4) = *(ushort *)(iVar1 + 4) & 0xfffd;
    *(byte *)(param_1 + 4) = *(byte *)(param_1 + 4) & 0xfe;
  }
  return;
}



/* ======================================================================
 * 0001237a  lmc_req_collect_completed
 * ====================================================================== */

undefined8 lmc_req_collect_completed(uint param_1,int param_2)

{
  byte bVar1;
  int iVar2;
  int iVar3;
  int iVar4;
  int extraout_r1;
  int extraout_r1_00;
  int iVar5;
  uint uVar6;
  uint uVar7;
  uint uVar8;
  
  iVar2 = DAT_000124e0;
  iVar5 = DAT_000124e0 + DAT_0001250c;
  for (uVar6 = 0; uVar6 < param_1; uVar6 = uVar6 + 1 & 0xff) {
    bVar1 = *(byte *)(iVar5 + 2);
    fw_div_scaled(bVar1 + 0x1d,0x1e);
    if (*(char *)(extraout_r1 + iVar2 + 0x4c04) != '\x01') break;
    if (bVar1 == 0) {
      *(undefined1 *)(iVar5 + 2) = 0x1d;
    }
    else {
      *(byte *)(iVar5 + 2) = bVar1 - 1;
    }
    iVar4 = *(int *)((uint)*(byte *)(iVar5 + 2) * 4 + iVar2 + 0x4b8c);
    *(int *)(param_2 + uVar6 * 4) = iVar4;
    iVar3 = DAT_000124e0;
    *(short *)(iVar4 + 2) = (short)DAT_00012510;
    *(undefined1 *)((uint)*(byte *)(iVar5 + 2) + iVar3 + 0x4c04) = 0;
    *(undefined4 *)((uint)*(byte *)(iVar5 + 2) * 4 + iVar2 + 0x4b8c) = 0;
  }
  uVar8 = param_1 - uVar6 & 0xff;
  uVar7 = 0;
  do {
    if (uVar8 == 0) break;
    __udivsi3((uint)*(byte *)(iVar5 + 2) + uVar7 + 0x1e,0x1e);
    bVar1 = *(byte *)(extraout_r1_00 + iVar2 + 0x4c04);
    if ((int)((uint)bVar1 << 0x1e) < 0) {
      uVar8 = uVar8 - 1 & 0xff;
      *(byte *)(extraout_r1_00 + iVar2 + 0x4c04) = bVar1 | 8;
    }
    uVar7 = uVar7 + 1 & 0xff;
  } while (uVar7 < 0x1e);
  return CONCAT44(uVar6,uVar6);
}



/* ======================================================================
 * 00012434  measure_next_pending_slot
 * ====================================================================== */

uint measure_next_pending_slot(void)

{
  int iVar1;
  uint uVar2;
  undefined1 extraout_r1;
  uint uVar3;
  
  iVar1 = DAT_000124fc;
  uVar2 = (uint)*(byte *)(DAT_000124fc + 3);
  uVar3 = 0xff;
  if ((*(byte *)(DAT_000124fc + 2) != uVar2) && (*(char *)(uVar2 + DAT_000124fc + -0x1c) == '\x01'))
  {
    *(undefined1 *)(uVar2 + DAT_000124fc + -0x1c) = 3;
    __udivsi3(*(byte *)(iVar1 + 3) + 1,0x1e);
    *(undefined1 *)(iVar1 + 3) = extraout_r1;
    uVar3 = uVar2;
  }
  return uVar3;
}



/* ======================================================================
 * 00012462  lmc_req_confirm_and_release
 * ====================================================================== */

void lmc_req_confirm_and_release(uint param_1,undefined2 *param_2)

{
  undefined4 uVar1;
  int iVar2;
  int iVar3;
  
  if (0x1d < param_1) {
    fw_assert(s_lmc_c_00012504,DAT_00012500 + 0x69,0x33);
  }
  iVar3 = DAT_000124e0;
  iVar2 = param_1 * 4 + DAT_000124e0;
  if (*(undefined2 **)(iVar2 + 0x4b8c) != param_2) {
    fw_assert(s_lmc_c_00012504,DAT_00012500 + 0x6e,0x34);
  }
  iVar3 = iVar3 + param_1;
  if (-1 < (int)((uint)*(byte *)(iVar3 + 0x4c04) << 0x1e)) {
    fw_assert(s_lmc_c_00012504,DAT_00012500 + 0x72,0x35);
  }
  if ((int)((uint)*(byte *)(iVar3 + 0x4c04) * 0x10000000) < 0) {
    *(undefined4 *)(iVar2 + 0x4b8c) = 0;
    uVar1 = DAT_00012510;
    *(undefined1 *)(iVar3 + 0x4c04) = 0;
    param_2[1] = (short)uVar1;
    *param_2 = 8;
    hif_send_msg_to_host(param_2);
    return;
  }
  *(undefined1 *)(param_2 + 2) = 0;
  lmc_req_slot_enqueue(param_2);
  *(undefined4 *)(iVar2 + 0x4b8c) = 0;
  *(undefined1 *)(iVar3 + 0x4c04) = 0;
  return;
}



/* ======================================================================
 * 00012514  phy_set_band_reg
 * ====================================================================== */

void phy_set_band_reg(ushort param_1)

{
  undefined1 *puVar1;
  undefined4 uVar2;
  
  *DAT_0001256c = param_1;
  puVar1 = DAT_00012570;
  if (((~param_1 & 0x22) == 0) || ((param_1 & 0x7f) == 0x12)) {
    uVar2 = 0x40;
  }
  else {
    uVar2 = 1;
  }
  *(undefined4 *)(DAT_00012570 + 8) = uVar2;
  puVar1[2] = 0xf;
  *puVar1 = 1;
  return;
}



/* ======================================================================
 * 0001253a  pas_pick_lowest_rate_from_mask
 * ====================================================================== */

void pas_pick_lowest_rate_from_mask(int param_1)

{
  undefined1 uVar1;
  uint uVar2;
  
  uVar2 = 0;
  do {
    if ((1 << uVar2 & *(uint *)(param_1 + 0x10)) != 0) break;
    if (uVar2 == 0x15) {
      uVar2 = 6;
      break;
    }
    uVar2 = uVar2 + 1 & 0xff;
  } while (uVar2 != 0);
  uVar1 = (undefined1)uVar2;
  *(undefined1 *)(param_1 + 0xc) = uVar1;
  if ((*(char *)(param_1 + 0xe) != '\0') && (3 < uVar2)) {
    uVar1 = 0;
  }
  *(undefined1 *)(param_1 + 0xd) = uVar1;
  return;
}



/* ======================================================================
 * 00012574  tx_ctx_pool_init
 * ====================================================================== */

/* tx_ctx_pool_init() -- builds the TX descriptor free list.  *** This is the
   authoritative source for sizeof(tx_ctx) and for the descriptor count. ***
   
     for (i = 0; i < 3; i++) {
         buf          = g_tx_bufs + i*0x400;      /* 1 KiB frame buffer each */
         ctx->pHdr80211  = buf + 0x40;            /* +0x1C */
         ctx->pCipherBuf = buf + 0x20;            /* +0xC4 */
         ctx->wResult    = 0xFF;                  /* +0x70 */
         ctx->pNextFree  = prev;                  /* +0x04 */
         prev = ctx;  ctx += 0x170;
     }
   
   So **sizeof(xr_tx_ctx) == 0x170 == 368 bytes** and there are only **THREE**
   descriptors in this pool, each with a 1 KiB frame buffer.
   
   An earlier revision put the struct at 208 (0xD0) bytes, which was wrong; the
   nested xr_tx_pas view at +0x54 happens to end exactly at 0xD0, and that
   coincidence was mistakenly treated as confirmation of the size.  The real
   struct runs to 0x170: tx_select_key_and_cipher alone touches +0x116, +0x118
   and +0x168.
   
   Note this pool is NOT the 30 host input buffers the startup indication
   advertises -- it is the internal set-frame / encryption pipeline, three deep.
   Also initialises the 24-bit IV/PN seed from fw_rand24_lfsr(). */

void tx_ctx_pool_init(void)

{
  undefined4 uVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  int iVar5;
  int iVar6;
  int iVar7;
  
  iVar2 = DAT_000125c8;
  uVar1 = DAT_000125c4;
  *(undefined4 *)(DAT_000125c8 + 0x10) = DAT_000125c4;
  *(undefined4 *)(iVar2 + 0x80) = uVar1;
  uVar4 = fw_rand24_lfsr();
  *DAT_000125cc = uVar4 & 0xffffff;
  iVar3 = DAT_000125d8;
  iVar2 = DAT_000125d4;
  uVar4 = 0;
  iVar7 = DAT_000125d0;
  iVar6 = 0;
  do {
    iVar5 = iVar7;
    iVar7 = uVar4 * 0x400 + iVar3 + iVar2;
    *(int *)(iVar5 + 0x1c) = iVar7 + 0x40;
    *(int *)(iVar5 + 0xc4) = iVar7 + 0x20;
    *(undefined2 *)(iVar5 + 0x70) = 0xff;
    *(int *)(iVar5 + 4) = iVar6;
    uVar4 = uVar4 + 1;
    iVar7 = iVar5 + 0x170;
    iVar6 = iVar5;
  } while (uVar4 < 3);
  *(int *)(DAT_000125d0 + -4) = iVar5;
  return;
}



/* ======================================================================
 * 000125dc  measure_ctl_reset
 * ====================================================================== */

void measure_ctl_reset(void)

{
  *DAT_000129c8 = 0;
  *DAT_000129cc = 0;
  *DAT_000129d0 = 0;
  return;
}



/* ======================================================================
 * 000125ee  measure_state_init
 * ====================================================================== */

void measure_state_init(void)

{
  int iVar1;
  
  iVar1 = DAT_000129d4;
  *(undefined1 *)(DAT_000129d4 + 7) = 0;
  *(undefined2 *)(iVar1 + 0xe) = 0x20;
  return;
}



/* ======================================================================
 * 000125fa  measure_set_channel
 * ====================================================================== */

void measure_set_channel(uint param_1)

{
  int iVar1;
  
  *DAT_000129d8 = *DAT_000129d8 | 0x40000;
  phy_set_band_reg(1);
  *(short *)(DAT_000129d4 + 2) = (short)param_1;
  iVar1 = DAT_000129dc;
  *(bool *)DAT_000129dc = 0xe < param_1;
  *(undefined1 *)(iVar1 + 2) = 0;
  syn_scan_program_channel();
  return;
}



/* ======================================================================
 * 0001262c  measure_finish_state7
 * ====================================================================== */

void measure_finish_state7(int param_1)

{
  char cVar1;
  
  if (*DAT_000129e0 == '\a') {
    if (param_1 == 0) {
      cVar1 = '\b';
    }
    else {
      cVar1 = '\t';
    }
    *DAT_000129e0 = cVar1;
    evt_flags_set(DAT_000129e4,0x2000);
  }
  return;
}



/* ======================================================================
 * 0001264e  measure_state_machine_tail
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00012656) */
/* WARNING: Removing unreachable block (ram,0x00012656) */

void measure_state_machine_tail(void)

{
  byte *pbVar1;
  
                    /* WARNING: Could not recover jumptable at 0x00012656. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (*DAT_000129e0 - 3 < (uint)DAT_0001265a) {
    pbVar1 = (byte *)(*DAT_000129e0 + 0x12658);
  }
  else {
    pbVar1 = (byte *)(DAT_0001265a + 0x1265b);
  }
  (*(code *)((uint)*pbVar1 * 2 + 0x1265b))();
  return;
}



/* ======================================================================
 * 00012680  measure_state_machine
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00012656) */
/* WARNING: Removing unreachable block (ram,0x00012656) */

void measure_state_machine(void)

{
  byte *pbVar1;
  
  if (*DAT_000129e0 == 0) {
    return;
  }
                    /* WARNING: Could not recover jumptable at 0x00012656. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (*DAT_000129e0 - 3 < (uint)DAT_0001265a) {
    pbVar1 = (byte *)(*DAT_000129e0 + 0x12658);
  }
  else {
    pbVar1 = (byte *)(DAT_0001265a + 0x1265b);
  }
  (*(code *)((uint)*pbVar1 * 2 + 0x1265b))();
  return;
}



/* ======================================================================
 * 0001268c  measure_state_dispatch_args
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x000126a8) */
/* WARNING: Removing unreachable block (ram,0x000126a8) */

void measure_state_dispatch_args(void)

{
  uint uVar1;
  int iVar2;
  
  uVar1 = (uint)*DAT_000129e0;
                    /* WARNING: Could not recover jumptable at 0x000126a8. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_000126ac <= uVar1) {
    uVar1 = (uint)DAT_000126ac;
  }
  iVar2 = (uint)*(byte *)(uVar1 + 0x126ad) * 2;
  (*(code *)(iVar2 + 0x126ad))
            (*(undefined2 *)(DAT_000129ec + 0x12),*(undefined2 *)(DAT_000129d4 + 2),DAT_000129ec,
             iVar2,*(undefined2 *)(DAT_000129ec + 0x20));
  return;
}



/* ======================================================================
 * 000128ea  measure_arm_dwell_timer
 * ====================================================================== */

void measure_arm_dwell_timer(void)

{
  char cVar1;
  char *pcVar2;
  undefined2 *puVar3;
  int iVar4;
  undefined4 uVar5;
  undefined8 uVar6;
  longlong lVar7;
  
  puVar3 = DAT_000129ec;
  pcVar2 = DAT_000129e0;
  cVar1 = *DAT_000129e0;
  if (cVar1 == '\x01') {
    *DAT_000129e0 = '\x06';
    uVar5 = fw_rand_below(*puVar3);
    timer_entry_init(DAT_000129e8,DAT_000129f8,0);
    timer_start(DAT_000129e8,uVar5);
    return;
  }
  if (cVar1 == '\x05') {
    lVar7 = tsf_read(2);
    uVar5 = 0;
    *(longlong *)(puVar3 + 0x10) = lVar7 - *(longlong *)(puVar3 + 0xc);
  }
  else {
    if (cVar1 != '\x06') {
      return;
    }
    if ((*(char *)((int)DAT_000129ec + 0x29) == '\x01') &&
       (iVar4 = syn_scan_begin_request(DAT_000129ec + 0x14), iVar4 == 0)) {
      uVar6 = tsf_read(2);
      *pcVar2 = '\x05';
      *(undefined8 *)(puVar3 + 0xc) = uVar6;
      return;
    }
    uVar5 = 1;
  }
  *(undefined4 *)(puVar3 + 4) = uVar5;
  measure_emit_complete();
  return;
}



/* ======================================================================
 * 00012964  measure_is_state5
 * ====================================================================== */

undefined4 measure_is_state5(void)

{
  if (*DAT_000129e0 == '\x05') {
    return 1;
  }
  return 0;
}



/* ======================================================================
 * 00012974  ind_080D_tx_trace
 * ====================================================================== */

void ind_080D_tx_trace(ushort *param_1)

{
  int iVar1;
  short *psVar2;
  undefined8 uVar3;
  
  iVar1 = measure_is_state5();
  if ((iVar1 != 0) &&
     (psVar2 = (short *)hif_alloc_msg_to_host(*param_1 + 0x14), psVar2 != (short *)0x0)) {
    iVar1 = *(int *)(DAT_000129fc + 0x14);
    psVar2[1] = (short)DAT_00012a00;
    *psVar2 = *param_1 + 0x14;
    psVar2[3] = (short)((iVar1 << 0x16) >> 0x16);
    *(undefined1 *)(psVar2 + 2) = 0;
    uVar3 = tsf_read(2);
    *(undefined8 *)(psVar2 + 4) = uVar3;
    fw_memcpy_words(psVar2 + 8,param_1,*param_1 >> 2);
    hif_send_msg_to_host(psVar2);
  }
  return;
}



/* ======================================================================
 * 00012b08  measure_setup_by_type
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00012b28) */
/* WARNING: Removing unreachable block (ram,0x00012b28) */

void measure_setup_by_type(int param_1)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  fw_memzero((void *)(DAT_00012da8 + -0x14),0x68);
  iVar1 = DAT_00012da8;
  iVar2 = DAT_00012da8 + -0x14;
  *(undefined1 *)(DAT_00012da8 + -0xf) = *(undefined1 *)(param_1 + 9);
  *(undefined4 *)(iVar1 + -0xc) = 0;
  uVar3 = (uint)*(byte *)(param_1 + 9);
                    /* WARNING: Could not recover jumptable at 0x00012b28. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_00012b2c <= uVar3) {
    uVar3 = (uint)DAT_00012b2c;
  }
  (*(code *)((uint)*(byte *)(uVar3 + 0x12b2d) * 2 + 0x12b2d))(iVar2,0,0x20);
  return;
}



/* ======================================================================
 * 00012bc0  measure_start
 * ====================================================================== */

longlong measure_start(int param_1,uint param_2,undefined4 param_3,undefined4 param_4)

{
  byte bVar1;
  char *pcVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  uint uVar7;
  char cVar8;
  bool bVar9;
  char cVar10;
  undefined8 uVar11;
  
  uVar6 = 0;
  uVar7 = 0;
  measure_setup_by_type();
  if ((*DAT_00012db8 & 0x100) != 0 || *DAT_00012dbc != '\0') {
    return CONCAT44(param_2,1);
  }
  uVar5 = 0;
  do {
    iVar4 = uVar5 * 0x3b0 + DAT_00012dac + -0x18;
    uVar3 = *(uint *)(iVar4 + 0x1c);
    if ((((uVar3 & 0x21) == 1) && (*(char *)(uVar5 * 0x104 + DAT_00012dc0 + 0x59) != '\0')) &&
       (-1 < (int)(uVar3 << 0x12))) {
      *(uint *)(iVar4 + 0x1c) = uVar3 | 0x2000;
      lmc_recompute_vif_roles();
    }
    uVar5 = uVar5 + 1;
  } while (uVar5 < 2);
  if (((*(char *)(param_1 + 9) == '\0') || (*(char *)(param_1 + 9) == '\x01')) &&
     (*(char *)(param_1 + 0x11) == '\0')) {
    uVar6 = *(uint *)(param_1 + 0x18);
    uVar7 = *(uint *)(param_1 + 0x1c);
  }
  cVar8 = uVar6 == 0;
  u64_cmp(uVar6,uVar7,0,0,param_2,param_3,param_4);
  if (cVar8 != '\0') goto LAB_00012cae;
  uVar11 = tsf_read(2);
  uVar3 = (uint)((ulonglong)uVar11 >> 0x20);
  uVar5 = (uint)uVar11;
  param_2 = uVar6 - uVar5;
  cVar10 = uVar3 < uVar7 || uVar7 - uVar3 < (uint)(uVar5 <= uVar6);
  iVar4 = (uVar7 - uVar3) - (uint)(uVar5 > uVar6);
  cVar8 = uVar6 == 0;
  u64_cmp(uVar6,uVar7,0,0);
  u64_cmp(uVar6,uVar7,uVar5,uVar3);
  if (cVar10 == '\0') {
LAB_00012c8e:
    bVar1 = 1;
  }
  else {
    bVar9 = true;
    u64_cmp(param_2,iVar4,DAT_00012dc4);
    if ((bool)cVar10 && !bVar9) goto LAB_00012c8e;
    bVar1 = 0;
  }
  pcVar2 = DAT_00012dbc;
  if ((bool)(cVar8 == '\0' & bVar1)) {
    *DAT_00012dbc = '\t';
    pcVar2[1] = '\x02';
    evt_flags_set(DAT_00012dc8,0x2000);
    return CONCAT44(param_2,2);
  }
LAB_00012cae:
  pcVar2 = DAT_00012dbc;
  *DAT_00012dbc = '\x01';
  pcVar2[1] = '\x01';
  evt_flags_set(DAT_00012dc8,0x2000);
  return (ulonglong)param_2 << 0x20;
}



/* ======================================================================
 * 00012cc2  task_measure_complete
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00012d16) */
/* WARNING: Removing unreachable block (ram,0x00012d16) */

void task_measure_complete(void)

{
  char cVar1;
  char cVar2;
  int iVar3;
  char *pcVar4;
  uint uVar5;
  
  pcVar4 = DAT_00012dbc;
  iVar3 = DAT_00012da8;
  if (DAT_00012dbc[1] != '\x02') {
    dbg_set_period(*DAT_00012db4);
    uVar5 = (uint)*(byte *)(iVar3 + -0xf);
                    /* WARNING: Could not recover jumptable at 0x00012d16. Too many branches */
                    /* WARNING: Treating indirect jump as call */
    if (DAT_00012d1a <= uVar5) {
      uVar5 = (uint)DAT_00012d1a;
    }
    (*(code *)((uint)*(byte *)(uVar5 + 0x12d1b) * 2 + 0x12d1b))();
    return;
  }
  cVar1 = *DAT_00012dbc;
  if (cVar1 == '\0') goto LAB_00012d56;
  cVar2 = *(char *)(DAT_00012da8 + -0xf);
  if ((cVar2 == '\0') || (cVar2 == '\x01')) {
    measure_state_machine_tail();
    goto LAB_00012d56;
  }
  if (cVar2 != '\x02') goto LAB_00012d56;
  if (cVar1 == '\x01') {
LAB_00012cf0:
    timer_cancel(DAT_00012dcc);
  }
  else if (cVar1 == '\x05') {
    syn_scan_stop_clear_flag();
  }
  else if (cVar1 == '\x06') goto LAB_00012cf0;
  *(undefined4 *)(iVar3 + -0xc) = 1;
  measure_emit_complete();
LAB_00012d56:
  pcVar4[1] = '\0';
  return;
}



/* ======================================================================
 * 00012d5c  wsm_h_0F_set_state_signal
 * ====================================================================== */

/* WSM command 0x000F backend -- STOP / ABORT 802.11k MEASUREMENT.
   
     g_measure_ctl[1] = 2;                  /* 0x04001FC9 */
     signal(g_event_obj, 0x2000);           /* wake the measurement task */
     return 0;
   
   Paired with command 0x000E (start measure), whose backend measure_start
   (0x00012BC0) drives the SAME control block at g_measure_ctl (0x04001FC8):
   
     early-out with status 1 if g_measure_ctl[0] != 0        (already running)
     on the normal start path:  [0] = 1, [1] = 1, signal 0x2000, return 0
     on the refuse path:        [0] = 9, [1] = 2, signal 0x2000, return 2
   
   So byte[0] is the measurement state (0 idle / 1 running / 9 refused) and
   byte[1] is the completion code, where 2 means "terminated".  0x000F sets
   only byte[1] and wakes the task, i.e. it cancels an in-progress
   measurement and lets it report completion -- the STOP half of the 11k
   start/stop pair.
   
   Neither driver issues it.  The vendor tree has wsm_start_measure_requset
   for 0x000E but no stop counterpart. */

undefined4 wsm_h_0F_set_state_signal(void)

{
  *(undefined1 *)(DAT_00012dbc + 1) = 2;
  evt_flags_set(DAT_00012dc8,0x2000);
  return 0;
}



/* ======================================================================
 * 00012d70  fw_rand_below
 * ====================================================================== */

undefined4 fw_rand_below(int param_1)

{
  undefined4 uVar1;
  undefined4 extraout_r1;
  
  if (param_1 == 0) {
    return 0;
  }
  uVar1 = fw_rand24_lfsr();
  __udivsi3(uVar1,param_1 << 10);
  return extraout_r1;
}



/* ======================================================================
 * 00012d88  measure_emit_complete
 * ====================================================================== */

void measure_emit_complete(void)

{
  if (*(int *)(DAT_00012da8 + -0xc) != 0) {
    *(undefined4 *)(DAT_00012da8 + -0xc) = 1;
  }
  ind_0807_measure_complete(DAT_00012da8 + -0x14);
  *DAT_00012dbc = 0;
  return;
}



/* ======================================================================
 * 00012e28  ps_send_pm_complete_ind
 * ====================================================================== */

void ps_send_pm_complete_ind
               (undefined1 *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined1 uVar1;
  uint local_10;
  undefined1 local_c;
  undefined3 uStack_b;
  
  *(undefined1 *)(DAT_000131cc + 8) = 0;
  local_10 = (uint)*(ushort *)(param_1 + 0xf8);
  if (local_10 == 0) {
    uVar1 = *param_1;
  }
  else {
    uVar1 = 2;
  }
  _local_c = CONCAT31((int3)((uint)param_4 >> 8),uVar1);
  ind_0809_set_pm_complete(param_1[3],&local_10);
  return;
}



/* ======================================================================
 * 00012e4e  vif_reset_all_state
 * ====================================================================== */

void vif_reset_all_state(uint param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  
  iVar5 = param_1 * 0x104 + DAT_000131cc + -0x20;
  *(undefined1 *)(iVar5 + 0x40) = 0;
  if ((*(byte *)(iVar5 + 0x58) & 1) != 0) {
    *(byte *)(iVar5 + 0x58) = *(byte *)(iVar5 + 0x58) & 0xfe;
    ps_send_pm_complete_ind((undefined1 *)(iVar5 + 0x40));
  }
  uVar2 = 0;
  *(undefined1 *)(iVar5 + 0x58) = 0;
  iVar1 = DAT_000131cc;
  do {
    if ((uVar2 != param_1) &&
       (iVar3 = uVar2 * 0x3b0 + DAT_000131d0, iVar4 = iVar3 + 0x18,
       (*(uint *)(iVar3 + 0x1c) & 9) != 0)) break;
    uVar2 = uVar2 + 1;
    iVar4 = 0;
  } while (uVar2 < 2);
  if ((iVar4 == 0) && (*(char *)(DAT_000131cc + 10) != '\0')) {
    *DAT_000131d4 = *DAT_000131d4 & 0xffffff7f;
    syn_scan_stop_dup();
    measure_state_machine();
    *(undefined1 *)(iVar1 + 10) = 0;
  }
  *(undefined1 *)(iVar5 + 0x124) = 0;
  mac_set_ps_bit(param_1,0);
  *(undefined2 *)(iVar5 + 0x5a) = 0;
  *(undefined2 *)(iVar5 + 0x44) = 0;
  *(undefined2 *)(iVar5 + 0x46) = 0;
  *(undefined1 *)(iVar5 + 0x59) = 0;
  *(undefined1 *)(iVar5 + 0x41) = 0;
  *(undefined1 *)(iVar5 + 0xfc) = 0;
  *(undefined1 *)(iVar5 + 0x13d) = 0;
  *(undefined4 *)(iVar5 + 0x5c) = 0;
  *(undefined1 *)(iVar5 + 0x50) = 0;
  *(undefined4 *)(iVar5 + 0x108) = 0;
  *(undefined2 *)(iVar5 + 0x10c) = 0;
  *(undefined4 *)(iVar5 + 0x54) = 0;
  *(undefined1 *)(iVar5 + 0x115) = 0;
  *(undefined2 *)(iVar5 + 0x138) = 0;
  timer_cancel(iVar5 + 0x84);
  timer_cancel(iVar5 + 0xd4);
  timer_cancel(iVar5 + 0x98);
  timer_cancel(iVar5 + 0xac);
  timer_cancel(iVar5 + 0x70);
  timer_cancel(iVar5 + 0xc0);
  timer_cancel(iVar5 + 0xe8);
  iVar5 = param_1 * 0x3b0 + DAT_000131d0;
  *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) & 0xffffdfff;
  vif_ps_exit_cleanup(param_1);
  timer_cancel(iVar5 + 0x184);
  timer_cancel(iVar5 + 0x198);
  *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) & 0xcfffffff;
  iVar5 = param_1 * 0x98 + DAT_000131d8;
  *(byte *)(iVar5 + 0x492) = *(byte *)(iVar5 + 0x492) & 0xfc;
  *(undefined1 *)(iVar5 + 0x493) = 0;
  lmc_recompute_vif_roles();
  return;
}



/* ======================================================================
 * 00012f7e  ps_try_enter_sleep_all
 * ====================================================================== */

void ps_try_enter_sleep_all(void)

{
  int iVar1;
  uint *puVar2;
  short sVar3;
  uint uVar4;
  ushort uVar5;
  ushort uVar6;
  int iVar7;
  ushort uVar8;
  
  puVar2 = DAT_000131d4;
  iVar1 = DAT_000131cc;
  if (*(char *)(DAT_000131cc + 10) == '\x03') {
    uVar4 = 0;
    do {
      iVar7 = uVar4 * 0x104 + DAT_000131cc + -0x20;
      if ((*(char *)(iVar7 + 0x41) != '\0') && (*(char *)(iVar7 + 0x40) != '\x01')) {
        *(undefined1 *)(iVar7 + 0xfc) = 0;
        *(byte *)(iVar7 + 0x58) = *(byte *)(iVar7 + 0x58) | 4;
        ps_reevaluate_all();
        return;
      }
      uVar4 = uVar4 + 1 & 0xff;
    } while (uVar4 < 2);
    *(undefined1 *)(DAT_000131cc + 10) = 4;
    *puVar2 = *puVar2 | 0x80;
  }
  else if (*(char *)(DAT_000131cc + 10) != '\x04') {
    return;
  }
  sVar3 = 0;
  uVar6 = 0;
  uVar8 = 0;
  uVar4 = 0;
  do {
    iVar7 = uVar4 * 0x104 + DAT_000131cc + -0x20;
    if (*(char *)(iVar7 + 0x41) != '\0') {
      if ((int)((uint)*(ushort *)(iVar7 + 0x44) << 0x19) < 0) {
        *(ushort *)(iVar7 + 0x44) = *(ushort *)(iVar7 + 0x44) & 0xffbf;
        *(ushort *)(iVar7 + 0x46) = *(ushort *)(iVar7 + 0x46) | 0x40;
      }
      uVar6 = uVar6 | *(ushort *)(iVar7 + 0x44);
      uVar8 = *(short *)(iVar7 + 0x13a) + uVar8;
      if (*(short *)(iVar7 + 0x138) != 0) {
        sVar3 = *(short *)(iVar7 + 0x138);
      }
    }
    uVar4 = uVar4 + 1 & 0xff;
  } while (uVar4 < 2);
  if ((int)(*DAT_000131d4 << 0x13) < 0) {
    uVar5 = 0x111;
  }
  else {
    uVar5 = 0x131;
  }
  if ((*(short *)(DAT_000131dc + 10) == 0) && (((uVar6 & uVar5) == 0 || (4 < uVar8)))) {
    *(undefined1 *)(iVar1 + 10) = 5;
    syn_scan_complete_if_scanning(sVar3);
    measure_finish_state7(0);
    return;
  }
  return;
}



/* ======================================================================
 * 00013054  ps_clear_flag8_and_reeval
 * ====================================================================== */

void ps_clear_flag8_and_reeval(int param_1)

{
  ushort uVar1;
  uint uVar2;
  
  uVar2 = *(ushort *)(param_1 + 4) & 0xfffffff7;
  uVar1 = (ushort)uVar2;
  *(ushort *)(param_1 + 4) = uVar1;
  if (-1 < (int)(uVar2 << 0x1b)) {
    if (*(byte *)(DAT_000131cc + 10) < 3) {
      if (*(short *)(DAT_000131dc + 10) == 0) {
        ps_reevaluate_all();
        return;
      }
      *(ushort *)(param_1 + 4) = uVar1 | 8;
      return;
    }
    ps_try_enter_sleep_all();
  }
  return;
}



/* ======================================================================
 * 00013084  ps_pspoll_retry_timer
 * ====================================================================== */

void ps_pspoll_retry_timer(char *param_1)

{
  char cVar1;
  
  cVar1 = param_1[3];
  *(ushort *)(param_1 + 4) = *(ushort *)(param_1 + 4) & 0xffbf;
  if ((*(char *)(DAT_000131cc + 10) != '\x05') && (*param_1 == '\x01')) {
    if (1 < (byte)param_1[0xd4]) {
      event_send_ps_mode_error(cVar1,1);
      ps_release_radio_if_all_idle(cVar1);
      return;
    }
    param_1[0xd4] = param_1[0xd4] + 1;
    tx_send_ps_poll(cVar1);
  }
  return;
}



/* ======================================================================
 * 000130c4  ps_send_pspoll_or_defer
 * ====================================================================== */

void ps_send_pspoll_or_defer(int param_1)

{
  if (*(int *)(param_1 + 0x14) != 0) {
    if (-1 < *DAT_000131d4 << 0x18) {
      *(undefined2 *)(param_1 + 0xce) = 0;
      *(undefined1 *)(param_1 + 0xd4) = 0;
      tx_send_ps_poll(*(undefined1 *)(param_1 + 3));
      return;
    }
    *(ushort *)(param_1 + 4) = *(ushort *)(param_1 + 4) & 0xffbf;
    *(ushort *)(param_1 + 6) = *(ushort *)(param_1 + 6) | 0x40;
  }
  return;
}



/* ======================================================================
 * 000130f6  ps_clear_and_release
 * ====================================================================== */

void ps_clear_and_release(int param_1)

{
  uint uVar1;
  
  uVar1 = (uint)*(ushort *)(param_1 + 4);
  if ((int)(uVar1 << 0x1d) < 0) {
    *(short *)(param_1 + 4) = (short)(uVar1 & 0xffffffdb);
    if (-1 < (int)((uVar1 & 0xffffffdb) << 0x17)) goto LAB_00013128;
  }
  else if (-1 < (int)(uVar1 << 0x17)) {
    return;
  }
  *(ushort *)(param_1 + 4) = *(ushort *)(param_1 + 4) & 0xfeff;
  if (3 < *(byte *)(DAT_000131cc + 10)) {
    ps_try_enter_sleep_all();
    return;
  }
LAB_00013128:
  ps_release_radio_if_all_idle(*(undefined1 *)(param_1 + 3));
  return;
}



/* ======================================================================
 * 00013130  dbg_accumulate_wake_stats
 * ====================================================================== */

void dbg_accumulate_wake_stats(void)

{
  undefined4 *puVar1;
  char cVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  undefined4 *puVar6;
  
  puVar1 = DAT_000131cc;
  cVar2 = *(char *)((int)DAT_000131cc + 0xb) + '\x01';
  puVar6 = DAT_000131cc + -8;
  *(char *)((int)DAT_000131cc + 0xb) = cVar2;
  if (cVar2 == '\x02') {
    *(undefined1 *)puVar6 = 0;
    *(undefined1 *)((int)puVar1 + -0x1e) = 0;
    *(undefined1 *)((int)puVar1 + -0x1f) = 0;
    dbg_take_and_clear_reg30();
    return;
  }
  uVar3 = dbg_take_and_clear_reg30();
  iVar4 = fw_read_timer();
  uVar5 = iVar4 - puVar1[10];
  if (*(char *)((int)puVar1 + 0xb) == '\x03') {
    puVar1[-3] = uVar5;
    puVar1[-1] = uVar5;
    puVar1[-4] = uVar3;
    puVar1[-6] = uVar3;
    puVar1[-2] = uVar5;
    puVar1[-5] = uVar3;
  }
  else {
    puVar1[-2] = puVar1[-2] + uVar5;
    if (uVar5 < (uint)puVar1[-3]) {
      puVar1[-3] = uVar5;
    }
    else if ((uint)puVar1[-1] < uVar5) {
      puVar1[-1] = uVar5;
    }
    puVar1[-5] = puVar1[-5] + uVar3;
    if (uVar3 < (uint)puVar1[-6]) {
      puVar1[-6] = uVar3;
    }
    else if ((uint)puVar1[-4] < uVar3) {
      puVar1[-4] = uVar3;
    }
    if (*(char *)((int)puVar1 + 0xb) == -0x7e) {
      if ((int)((uint)*(byte *)(puVar1 + 0x41) << 0x1b) < 0) {
        puVar6 = DAT_000131cc + -8;
        *puVar1 = puVar1[0xb];
        func_0xfff0198c(3,puVar6,0x93);
      }
      *(undefined1 *)((int)puVar1 + 0xb) = 1;
      return;
    }
  }
  return;
}



/* ======================================================================
 * 000131e0  wsm_h_10_set_pm_impl
 * ====================================================================== */

undefined4 wsm_h_10_set_pm_impl(undefined4 *param_1)

{
  byte bVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  
  *(undefined1 *)(DAT_000135a4 + 4) = 2;
  bVar1 = *(byte *)(DAT_000135a8 + 10);
  uVar4 = (uint)bVar1;
  iVar3 = uVar4 * 0x104 + DAT_000135ac;
  if ((uVar4 < 3) && (*(char *)(iVar3 + 0x59) != '\0')) {
    iVar2 = uVar4 * 0x3b0 + DAT_000135a8;
    uVar4 = *(uint *)(iVar2 + 0x1c);
    if ((uVar4 & 0x21) == 1) {
      if (-1 < (int)(uVar4 << 0x12)) {
        *(uint *)(iVar2 + 0x1c) = uVar4 | 0x2000;
        lmc_recompute_vif_roles();
      }
      *(undefined1 *)(DAT_000135a4 + 0x7f) = 1;
      *(undefined4 *)(iVar3 + 0x124) = *param_1;
      iVar2 = DAT_000135ac;
      *(byte *)(DAT_000135ac + 0x29) = bVar1;
      *(char *)(iVar2 + 0x2b) = (char)((*(byte *)(iVar3 + 0x124) & 0x1f) >> 4);
      *(byte *)(iVar3 + 0x58) = *(byte *)(iVar3 + 0x58) | 1;
      ps_reevaluate_all();
      return 0;
    }
  }
  return 8;
}



/* ======================================================================
 * 0001325a  uapsd_setup_from_params
 * ====================================================================== */

void uapsd_setup_from_params(int param_1)

{
  ushort uVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  
  iVar4 = param_1 * 0x104 + DAT_000135ac;
  if (*(char *)(iVar4 + 0x40) == '\x01') {
    iVar2 = param_1 * 0x3b0 + DAT_000135a8;
    *(undefined1 *)(iVar4 + 0x42) = 0;
    *(ushort *)(iVar4 + 0x44) = *(ushort *)(iVar4 + 0x44) & 0xfffe;
    uVar1 = *(ushort *)(iVar2 + 0x130);
    *(ushort *)(iVar4 + 0x5a) = uVar1;
    if ((uVar1 & 0xfff) >> 8 == 0) {
      *(ushort *)(iVar4 + 0x5a) = uVar1 | (ushort)((uVar1 & 0xf) << 8);
    }
    uVar3 = (uint)*(ushort *)(iVar2 + 0x132) * 1000;
    if (uVar3 != 0) {
      uVar5 = 0;
      do {
        if (((uint)*(ushort *)(iVar4 + 0x5a) & 1 << uVar5) != 0) {
          if (3 < uVar5) {
            return;
          }
          *(ushort *)(iVar4 + 0x5a) = *(ushort *)(iVar4 + 0x5a) | 0x40;
          *(undefined1 *)(iVar4 + 0x53) = 1;
          *(char *)(iVar4 + 0x52) = (char)uVar5;
          *(uint *)(iVar4 + 0x60) = uVar3;
          *(uint *)(iVar4 + 100) = uVar3;
          *(uint *)(iVar4 + 0x6c) = (uint)*(ushort *)(iVar2 + 0x136) * 1000;
          uVar5 = (uint)*(ushort *)(iVar2 + 0x134) * 1000;
          *(uint *)(iVar4 + 0x68) = uVar5;
          if (uVar5 < uVar3) {
            *(uint *)(iVar4 + 0x68) = uVar3;
          }
          timer_start(iVar4 + 0x98);
          return;
        }
        uVar5 = uVar5 + 1 & 0xff;
      } while (uVar5 < 4);
    }
  }
  return;
}



/* ======================================================================
 * 000132f4  uapsd_timer_restart
 * ====================================================================== */

void uapsd_timer_restart(int param_1)

{
  int iVar1;
  int iVar2;
  
  iVar1 = param_1 * 0x104 + DAT_000135ac;
  if (*(byte *)(iVar1 + 0x53) < 2) {
    return;
  }
  if (*(byte *)(iVar1 + 0x53) == 2) {
    if ((uint)(*(int *)(iVar1 + 100) + *(int *)(iVar1 + 0x6c)) < *(uint *)(iVar1 + 0x60)) {
      iVar2 = *(uint *)(iVar1 + 0x60) - *(int *)(iVar1 + 0x6c);
      goto LAB_0001331e;
    }
  }
  iVar2 = *(int *)(iVar1 + 100);
LAB_0001331e:
  *(int *)(iVar1 + 0x60) = iVar2;
  timer_start(iVar1 + 0x98);
  return;
}



/* ======================================================================
 * 00013328  ps_mark_awake
 * ====================================================================== */

void ps_mark_awake(int param_1)

{
  int iVar1;
  int iVar2;
  
  iVar1 = DAT_000135ac;
  *(undefined4 *)(DAT_000135b0 + 0x3c) = 0;
  iVar2 = param_1 * 0x104 + iVar1;
  *(undefined4 *)(iVar1 + 0x24) = 0;
  *(undefined1 *)(iVar2 + 0x59) = 1;
  *(undefined1 *)(iVar2 + 0x13c) = 0;
  return;
}



/* ======================================================================
 * 00013348  ps_all_vifs_can_sleep
 * ====================================================================== */

undefined4 ps_all_vifs_can_sleep(int param_1)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  iVar1 = DAT_000135ac;
  uVar3 = 0;
  do {
    iVar4 = uVar3 * 0x104 + DAT_000135ac;
    if (*(char *)(iVar4 + 0x41) != '\0') {
      if (*(char *)(iVar4 + 0x115) == '\0') {
        ps_schedule_next_tbtt_wake(uVar3 & 0xff);
        if (*(char *)(iVar4 + 0x115) != '\0') goto LAB_0001337a;
      }
      else {
LAB_0001337a:
        iVar2 = fw_read_timer();
        if (0 < ((iVar2 + param_1) - *(int *)(iVar4 + 0x110)) + 0x1000) goto LAB_000133a4;
      }
      if (((*(char *)(DAT_000135a8 + 6) != '\0') || (*(short *)(iVar4 + 0x46) != 0)) ||
         ((int)((uint)*(ushort *)(iVar4 + 0x44) << 0x1c) < 0)) {
LAB_000133a4:
        *(undefined1 *)(iVar1 + 0x2a) = 6;
        return 0;
      }
    }
    uVar3 = uVar3 + 1;
    if (1 < uVar3) {
      return 1;
    }
  } while( true );
}



/* ======================================================================
 * 000133b6  ps_exit_sleep_and_resume
 * ====================================================================== */

undefined4
ps_exit_sleep_and_resume
          (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  *DAT_000135b4 = *DAT_000135b4 & 0xffffff7f;
  iVar1 = DAT_000135ac;
  if (*(char *)(DAT_000135ac + 0x2a) != '\0') {
    if (*(char *)(DAT_000135ac + 0x2a) == '\x06') {
      *(undefined1 *)(DAT_000135ac + 0x2a) = 1;
    }
    else {
      *(undefined1 *)(DAT_000135ac + 0x2a) = 0;
    }
    uVar3 = (uint)*(byte *)(iVar1 + 0x2d);
    iVar2 = uVar3 * 0x104 + DAT_000135ac;
    *(ushort *)(iVar2 + 0x44) = *(ushort *)(iVar2 + 0x44) | *(ushort *)(iVar2 + 0x46);
    iVar1 = DAT_000135b8;
    *(undefined2 *)(iVar2 + 0x46) = 0;
    if (*(int *)(iVar1 + 0x28) == 0) {
      mac_set_ps_bit(uVar3,1,param_3,param_4,param_4);
      ps_reevaluate_all();
      if (*(char *)(iVar2 + 0x40) == '\x01') {
        if ((*(ushort *)(iVar2 + 0x44) & 1) != 0) {
          *(ushort *)(iVar2 + 0x5a) = *(ushort *)(iVar2 + 0x5a) | 0x80;
          ps_send_pending_poll_or_qosnull(uVar3);
        }
        if ((int)((uint)*(ushort *)(iVar2 + 0x44) << 0x19) < 0) {
          *(undefined1 *)(iVar2 + 0x114) = 0;
          tx_send_ps_poll(uVar3);
        }
        return 1;
      }
    }
  }
  return 0;
}



/* ======================================================================
 * 00013510  ps_compute_intervals
 * ====================================================================== */

void ps_compute_intervals(uint param_1,int param_2)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  uint *puVar5;
  int iVar6;
  
  iVar6 = param_1 * 0x3b0 + DAT_000135a8;
  iVar2 = param_1 * 0x104 + DAT_000135ac;
  puVar5 = (uint *)(param_1 * 0x3b0 + DAT_000135a8 + 0x118);
  if (*(byte *)(param_2 + 1) == 0) {
    uVar3 = *puVar5;
    uVar4 = uVar3;
    if ((uVar3 <= DAT_000135bc) && (uVar4 = DAT_000135bc, (uint)((int)DAT_000135bc >> 1) <= uVar3))
    {
      uVar4 = uVar3 << 1;
    }
  }
  else {
    uVar4 = (uint)*(byte *)(param_2 + 1) * 500;
  }
  *(uint *)(iVar2 + 0x118) = uVar4;
  *(uint *)(iVar2 + 0x11c) = uVar4;
  if ((1 << (param_1 & 0xff) & (uint)*DAT_000135c0) != 0) {
    *(undefined4 *)(iVar2 + 0x118) = DAT_000135c4;
  }
  if (*(byte *)(param_2 + 2) == 0) {
    iVar1 = 4000;
  }
  else {
    iVar1 = (uint)*(byte *)(param_2 + 2) * 500;
  }
  *(int *)(iVar2 + 0x120) = iVar1;
  iVar1 = DAT_000135ac;
  *(uint *)(iVar2 + 0x128) = (uint)*(byte *)(param_2 + 3) * 500;
  *(undefined4 *)(iVar1 + 0x30) = 16000;
  *(uint *)(iVar2 + 300) = *puVar5 * (uint)*(ushort *)(iVar6 + 0x378) >> 1;
  return;
}



/* ======================================================================
 * 000135c8  event_send_ps_mode_error
 * ====================================================================== */

void event_send_ps_mode_error(int param_1,undefined4 param_2)

{
  int iVar1;
  undefined4 local_10;
  undefined4 local_c;
  
  iVar1 = param_1 * 0x104 + DAT_00013648;
  if (*(char *)(iVar1 + 0x13c) == '\0') {
    *(undefined1 *)(iVar1 + 0x13c) = 1;
    local_10 = 7;
    local_c = param_2;
    ind_0805_event_a(param_1,&local_10);
  }
  return;
}



/* ======================================================================
 * 000135f0  ps_backoff_interval_update
 * ====================================================================== */

void ps_backoff_interval_update(int param_1)

{
  ushort uVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  
  iVar2 = param_1 * 0x104 + DAT_00013648;
  uVar1 = *(ushort *)(iVar2 + 0x10e);
  if (uVar1 == 0) {
    if (*(uint *)(iVar2 + 300) <= *(uint *)(iVar2 + 0x54)) {
      return;
    }
    uVar4 = *(uint *)(iVar2 + 0x54) * 2;
    *(uint *)(iVar2 + 0x54) = uVar4;
    uVar3 = *(uint *)(iVar2 + 300);
    if (uVar4 < uVar3 || uVar4 - uVar3 == 0) goto LAB_0001363e;
  }
  else {
    if (uVar1 < 2) goto LAB_0001363e;
    if (uVar1 == 2) {
      *(uint *)(iVar2 + 0x54) = *(uint *)(iVar2 + 0x54) >> 1;
LAB_00013632:
      if (*(uint *)(iVar2 + 0x128) <= *(uint *)(iVar2 + 0x54)) goto LAB_0001363e;
    }
    else if (uVar1 < 3) goto LAB_00013632;
    uVar3 = *(uint *)(iVar2 + 0x128);
  }
  *(uint *)(iVar2 + 0x54) = uVar3;
LAB_0001363e:
  timer_start(iVar2 + 0x70,*(undefined4 *)(iVar2 + 0x54));
  return;
}



/* ======================================================================
 * 0001364c  edca_apply_queue_params
 * ====================================================================== */

/* edca_apply_queue_params(arg) -- WSM 0x0012 SET_TX_QUEUE_PARAMS backend.
   *** This closes the TXOP-budget chain end to end. ***
   
     memcpy(g_fw_ctx + 0x400 + queue*0xC, arg, 0xC);       /* queue = arg[0] */
     for (q = 0; q < 4; q++) {
         txop = *(u16 *)(g_fw_ctx + q*0xC + 0x408);
         *(u32 *)(g_fw_ctx + q*4 + 0x430) = txop * 0x20;    /* <== txop * 32 */
         if (txop) any = 1;
     }
     g_txop_enabled = any;
     if (any) { txop_budget_reload(); timer_start(...); }
   
   So the `+0x430` value that `txop_budget_check` (`0x0000828C`) compares airtime
   against is written **here**, as `txop_limit * 32`, from the host's
   SET_TX_QUEUE_PARAMS payload.  That confirms every link in the previously
   documented chain from firmware side:
   
     host txop_limit -> *32 -> g_fw_ctx[q*4 + 0x430] -> txop_budget_check
       -> closes the A-MPDU when (limit - used) < pas->dwAirtimeUs
   
   and it confirms the budget is **per-AC with exactly 4 queues**, not per-vif -- so
   it is a different index space from the 0x98 per-vif block (Corrections #10).
   `limit == 0` disables the whole mechanism, and the `any` flag above means the
   reload timer does not even run when all four are zero.
   
   Note mainline sends `WSM_TX_QUEUE_SET(..., 0, 0, 0)` -- ackPolicy 0,
   allowedMediumTime 0, maxTransmitLifetime 0 -- for all four queues, so from *this*
   command the TXOP limits are all zero and the mechanism is off.  The non-zero
   values come from `0x0013 SET_EDCA_PARAMS` instead (edca_apply_params, 0x000136A6,
   which writes the per-vif 0x98 block at +0x4CC).  Worth keeping straight: two
   different commands, two different tables, and only one of them carries the AP's
   WMM TXOP. */

void edca_apply_queue_params(byte *param_1)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  int iVar4;
  
  iVar1 = DAT_00013704;
  iVar4 = 0;
  fw_memcpy((void *)((uint)*param_1 * 0xc + DAT_00013704 + 0x400),param_1,0xc);
  uVar2 = 0;
  do {
    uVar3 = (uint)*(ushort *)(uVar2 * 0xc + iVar1 + 0x408);
    *(uint *)(uVar2 * 4 + iVar1 + 0x430) = uVar3 * 0x20;
    if (uVar3 != 0) {
      iVar4 = 1;
    }
    uVar2 = uVar2 + 1 & 0xff;
  } while (uVar2 < 4);
  *(int *)(DAT_00013708 + 0x10) = iVar4;
  if (iVar4 != 0) {
    txop_budget_reload();
    timer_start(DAT_00013710,DAT_0001370c);
  }
  return;
}



/* ======================================================================
 * 000136a6  edca_apply_params
 * ====================================================================== */

/* edca_apply_params(if_id, arg) -- WSM 0x0013 SET_EDCA_PARAMS backend.
   *** This function pins down the whole per-vif 0x98 EDCA block. ***
   
     memcpy(g_fw_ctx + if_id*0x98 + 0x4CC, arg, 0x2C);
     slot = arg->aifns[0..3]-derived value  ->  +0x4FC
     if (slot changed) { program MAC slot timing; pas_backoff_reset_all(if_id); }
   
   The 0x2C bytes map exactly onto cw1200's wsm_set_edca_params() wire order, which
   serialises params[3], params[2], params[1], params[0] for each field:
   
     +0x4CC  u16 cwmin[4]           (8 bytes)
     +0x4D4  u16 cwmax[4]           (8 bytes)
     +0x4DC  u8  aifns[4]           (4 bytes)  <- read for the slot-timing calc
     +0x4E0  u16 txop_limit[4]      (8 bytes)  <- THE AGGREGATE AIRTIME BUDGET
     +0x4E8  u32 max_rx_lifetime[4] (16 bytes) <- THE FRAME LIFETIME
   
   That resolves two fields this project had attributed to the wrong command:
   
   * **`+0x4E0` is the EDCA `txop_limit`,** read by txq_agg_airtime_budget
     (0x0000A05E).  The driver already multiplies by TXOP_UNIT(32) in WSM_EDCA_SET,
     so the stored value is microseconds.  cw1200 defaults are VO=47*32=1504,
     VI=94*32=3008, **BE=0, BK=0**, and cw1200_conf_tx() overwrites all four from
     mac80211's params->txop -- i.e. from the AP's WMM IE.  The index is an **AC**,
     not a TID.
   * **`+0x4E8` is `max_rx_lifetime`,** read by txq_set_frame_lifetime (0x0000D160).
     cw1200 sets it to **0xC8 = 200** for every queue (sta.c:64-67 defaults and
     sta.c:639 in conf_tx), so the lifetime is 200 << 10 = 204800 us ~ 205 ms.
     It is NOT zero.
   
   Do not confuse either with the 0x0012 SET_TX_QUEUE_PARAMS table at +0x400, whose
   `allowedMediumTime` feeds the *separate* per-AC budget at +0x430 and which
   mainline does leave at zero.  Two commands, two tables, two budgets. */

void edca_apply_params(int param_1,void *param_2)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  iVar2 = param_1 * 0x98 + DAT_00013704;
  fw_memcpy((void *)(iVar2 + 0x4cc),param_2,0x2c);
  iVar1 = DAT_00013708;
  iVar3 = (uint)*(byte *)(iVar2 + 0x4dd) + (*(byte *)(iVar2 + 0x4df) - 1) * 0x1000 +
          (uint)*(byte *)(iVar2 + 0x4de) * 0x100 + (uint)*(byte *)(iVar2 + 0x4dc) * 0x10 + -0x111;
  *(int *)(iVar2 + 0x4fc) = iVar3;
  if (*(short *)(iVar1 + -0x50) == 0) {
    if (*(int *)(DAT_00013714 + 4) != iVar3) {
      *(int *)(DAT_00013714 + 4) = iVar3;
      *(int *)(DAT_00013718 + 0x24) = iVar3;
      pas_backoff_reset_all(param_1);
    }
  }
  return;
}



/* ======================================================================
 * 0001371c  enc_ctx_alloc
 * ====================================================================== */

undefined1 * enc_ctx_alloc(void)

{
  int iVar1;
  int iVar2;
  undefined1 *puVar3;
  
  irq_disable_save();
  iVar1 = DAT_00013a74;
  puVar3 = *(undefined1 **)(DAT_00013a74 + 4);
  if (puVar3 != (undefined1 *)0x0) {
    *(undefined4 *)(DAT_00013a74 + 4) = *(undefined4 *)(puVar3 + 0x18);
    *puVar3 = 1;
    puVar3[1] = 0xff;
    iVar2 = *(int *)(iVar1 + 8) + 1;
    *(int *)(iVar1 + 8) = iVar2;
    *(int *)(puVar3 + 0x14) = iVar2;
  }
  irq_restore();
  return puVar3;
}



/* ======================================================================
 * 00013746  enc_ctx_free
 * ====================================================================== */

void enc_ctx_free(undefined1 *param_1)

{
  int iVar1;
  
  irq_disable_save();
  *param_1 = 0;
  iVar1 = DAT_00013a74;
  *(undefined4 *)(param_1 + 0x18) = *(undefined4 *)(DAT_00013a74 + 4);
  *(undefined1 **)(iVar1 + 4) = param_1;
  irq_restore();
  return;
}



/* ======================================================================
 * 00013760  enc_ctx_free_offset
 * ====================================================================== */

void enc_ctx_free_offset(int param_1)

{
  int iVar1;
  
  irq_disable_save();
  *(undefined1 *)(param_1 + -0xf4) = 0;
  iVar1 = DAT_00013a74;
  *(undefined4 *)(param_1 + -0xdc) = *(undefined4 *)(DAT_00013a74 + 4);
  *(undefined1 **)(iVar1 + 4) = (undefined1 *)(param_1 + -0xf4);
  irq_restore();
  return;
}



/* ======================================================================
 * 00013764  rx_crypto_complete
 * ====================================================================== */

void rx_crypto_complete(int param_1)

{
  ushort uVar1;
  bool bVar2;
  int *piVar3;
  undefined1 *puVar4;
  
  piVar3 = *(int **)(param_1 + 0x58);
  puVar4 = *(undefined1 **)(param_1 + 0x5c);
  uVar1 = *(ushort *)piVar3[7];
  bVar2 = false;
  if ((*(char *)(*piVar3 + 9) == '\x02') || (*(char *)(*piVar3 + 9) == '\x03')) {
    bVar2 = true;
  }
  rxfifo_release_slot(piVar3[5]);
  txbuf_freelist_push(piVar3);
  if (-1 < (int)((uint)uVar1 << 0x15)) {
    *puVar4 = 2;
    if (bVar2) {
      **(undefined4 **)(puVar4 + 0xf4) = puVar4 + 0xf4;
      mic_build_aad_rx();
    }
    else {
      rx_deliver_or_queue_mgmt(puVar4 + 0xf4);
    }
  }
  *DAT_00013a78 = *DAT_00013a78 & 0xfffffffe;
  evt_flags_set(DAT_00013a7c,0x80000);
  return;
}



/* ======================================================================
 * 000137cc  enc_submit_frame
 * ====================================================================== */

void enc_submit_frame(int param_1,int *param_2,undefined4 param_3,undefined4 param_4,
                     undefined4 param_5)

{
  undefined4 uVar1;
  int iVar2;
  
  *DAT_00013a78 = *DAT_00013a78 | 1;
  if (*(char *)(param_1 + 0x10) == '\0') {
    *(undefined1 *)(param_1 + 0x11f) = *(undefined1 *)((int)param_2 + 0x2b);
    *(short *)(param_1 + 0x11c) = (short)param_2[10];
    *(undefined1 *)(param_1 + 0x116) = *(undefined1 *)((int)param_2 + 0x22);
    *(undefined1 *)(param_1 + 0x117) = *(undefined1 *)((int)param_2 + 0x23);
    *(undefined1 *)(param_1 + 0x103) = *(undefined1 *)((int)param_2 + 0xf);
    *(int *)(param_1 + 0xf4) = param_1 + 0x1c;
  }
  iVar2 = *param_2;
  *(undefined1 *)(iVar2 + 0x30) = 0;
  *(undefined4 *)(iVar2 + 0x44) = param_3;
  *(int *)(iVar2 + 0x88) = param_1;
  uVar1 = DAT_00013a80;
  *(int **)(iVar2 + 0x84) = param_2;
  *(undefined4 *)(iVar2 + 0x50) = uVar1;
  *(undefined4 *)(iVar2 + 0x48) = param_4;
  *(undefined4 *)(iVar2 + 0x4c) = param_5;
  hif_submit_or_queue(iVar2 + 0x2c);
  return;
}



/* ======================================================================
 * 00013820  lmc_copy_msg_fields
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x000138ca) */
/* WARNING: Removing unreachable block (ram,0x000138ca) */

void lmc_copy_msg_fields(int param_1,int *param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  if (*(char *)(param_1 + 0x10) == '\0') {
    *(int *)(param_1 + 0xfc) = param_2[2];
    *(short *)(param_1 + 0x100) = (short)param_2[3];
    *(undefined1 *)(param_1 + 0x102) = *(undefined1 *)((int)param_2 + 0xe);
    *(undefined1 *)(param_1 + 0x11e) = *(undefined1 *)((int)param_2 + 0x2a);
    *(undefined4 *)(param_1 + 0x108) = 0;
    *(undefined1 *)(param_1 + 0x103) = *(undefined1 *)((int)param_2 + 0xf);
    *(uint *)(param_1 + 0x104) = param_2[4] | 0x40;
    *(int *)(param_1 + 0x110) = param_1 + 0x120;
    *(char *)(param_1 + 0x114) = (char)param_2[8];
    *(undefined1 *)(param_1 + 0x11f) = *(undefined1 *)((int)param_2 + 0x2b);
    *(short *)(param_1 + 0x11c) = (short)param_2[10];
    *(undefined1 *)(param_1 + 0x116) = *(undefined1 *)((int)param_2 + 0x22);
    *(undefined1 *)(param_1 + 0x117) = *(undefined1 *)((int)param_2 + 0x23);
    *(undefined1 *)(param_1 + 0x103) = *(undefined1 *)((int)param_2 + 0xf);
    *(void **)(param_1 + 0xf4) = (void *)(param_1 + 0x1c);
    fw_memcpy((void *)(param_1 + 0x1c),(void *)*param_2,0xd8);
    enc_submit_frame(param_1,param_2,*(undefined4 *)(param_1 + 0xc),param_2[7],param_2[6] & 0xffff,
                     param_3,param_4);
    **(ushort **)(param_1 + 0xc) = **(ushort **)(param_1 + 0xc) & (ushort)DAT_00013a84;
    *(int *)(param_1 + 0xc) = *(int *)(param_1 + 0xc) + param_2[6];
    *(int *)(param_1 + 0x10c) = *(int *)(param_1 + 0x10c) + param_2[6];
    return;
  }
  iVar1 = *param_2;
  uVar2 = (uint)*(byte *)(iVar1 + 9);
                    /* WARNING: Could not recover jumptable at 0x000138ca. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_000138ce <= uVar2) {
    uVar2 = (uint)DAT_000138ce;
  }
  iVar3 = (uint)*(byte *)(uVar2 + 0x138cf) * 2;
  (*(code *)(iVar3 + 0x138cf))
            (*(undefined4 *)(param_1 + 0xc),param_2[6] - *(int *)(iVar1 + 0x10),iVar1,iVar3,param_2,
             param_3,param_2[7] + *(int *)(iVar1 + 0x10));
  return;
}



/* ======================================================================
 * 0001394c  lmc_msg_release_slot
 * ====================================================================== */

void lmc_msg_release_slot(int param_1)

{
  ushort uVar1;
  char *pcVar2;
  uint uVar3;
  ushort *puVar4;
  uint uVar5;
  char *pcVar6;
  ushort uVar7;
  
  uVar1 = *(ushort *)(param_1 + 0x24);
  uVar7 = 0xff;
  puVar4 = *(ushort **)(param_1 + 0x1c);
  if ((*puVar4 & 0x8f) == 0x88) {
    uVar7 = *(ushort *)(param_1 + 0x26) & 0xf;
  }
  if ((uVar1 & 0xf) == 0) {
    pcVar2 = (char *)enc_ctx_alloc();
    if (pcVar2 == (char *)0x0) {
      uVar3 = 2;
      uVar5 = 0;
      pcVar2 = (char *)0x0;
      do {
        uVar3 = uVar3 - 1 & 0xff;
        pcVar6 = (char *)(uVar3 * DAT_00013a88 + DAT_00013a90 + DAT_00013a8c);
        if ((*pcVar6 == '\x01') && ((pcVar2 == (char *)0x0 || (*(uint *)(pcVar6 + 0x14) < uVar5))))
        {
          uVar5 = *(uint *)(pcVar6 + 0x14);
          pcVar2 = pcVar6;
        }
      } while (uVar3 != 0);
      if (pcVar2 != (char *)0x0) {
        enc_ctx_free();
      }
      pcVar2 = (char *)enc_ctx_alloc();
      if (pcVar2 == (char *)0x0) goto LAB_00013a66;
    }
    pcVar2[0x10] = '\0';
    *(ushort *)(pcVar2 + 4) = puVar4[5];
    *(ushort *)(pcVar2 + 6) = puVar4[6];
    *(ushort *)(pcVar2 + 8) = puVar4[7];
    *(ushort *)(pcVar2 + 2) = uVar1 >> 4;
    pcVar2[1] = (char)uVar7;
    *pcVar2 = '\x01';
    pcVar2[0x10c] = '\0';
    pcVar2[0x10d] = '\0';
    pcVar2[0x10e] = '\0';
    pcVar2[0x10f] = '\0';
    *(undefined2 *)(pcVar2 + 0x118) = *(undefined2 *)(param_1 + 0x24);
    *(undefined2 *)(pcVar2 + 0x11a) = *(undefined2 *)(param_1 + 0x26);
    *(char **)(pcVar2 + 0xc) = pcVar2 + 0x120;
LAB_00013a56:
    lmc_copy_msg_fields(pcVar2,param_1);
    return;
  }
  uVar3 = 2;
  do {
    uVar3 = uVar3 - 1 & 0xff;
    pcVar2 = (char *)(uVar3 * DAT_00013a88 + DAT_00013a90 + DAT_00013a8c);
    if (((((*pcVar2 == '\x01') && (*(ushort *)(pcVar2 + 2) == uVar1 >> 4)) &&
         ((byte)pcVar2[1] == uVar7)) &&
        ((*(ushort *)(pcVar2 + 4) == puVar4[5] && (*(ushort *)(pcVar2 + 6) == puVar4[6])))) &&
       (*(ushort *)(pcVar2 + 8) == puVar4[7])) {
      if ((ushort)((byte)pcVar2[0x10] + 1) != (uVar1 & 0xf)) {
        rxfifo_release_slot(*(undefined4 *)(param_1 + 0x14));
        txbuf_freelist_push(param_1);
        enc_ctx_free(pcVar2);
        return;
      }
      pcVar2[0x10] = (char)(uVar1 & 0xf);
      goto LAB_00013a56;
    }
  } while (uVar3 != 0);
LAB_00013a66:
  rxfifo_release_slot(*(undefined4 *)(param_1 + 0x14));
  txbuf_freelist_push(param_1);
  return;
}



/* ======================================================================
 * 00013a94  dup_cache_init
 * ====================================================================== */

void dup_cache_init(void)

{
  int iVar1;
  undefined4 uVar2;
  uint uVar3;
  int iVar4;
  
  uVar2 = DAT_00013b3c;
  iVar1 = DAT_00013b38;
  uVar3 = 0;
  do {
    iVar4 = uVar3 * 0xc;
    uVar3 = uVar3 + 1;
    *(short *)(iVar4 + iVar1 + 0x4792) = (short)uVar2;
  } while (uVar3 < 0x20);
  *(undefined4 *)(DAT_00013b40 + 0xc) = 0;
  return;
}



/* ======================================================================
 * 00013ab8  dup_cache_invalidate_by_mac
 * ====================================================================== */

void dup_cache_invalidate_by_mac(uint param_1,short *param_2)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  iVar1 = DAT_00013b38;
  uVar2 = 0;
  do {
    iVar3 = uVar2 * 0xc + iVar1;
    if ((((*param_2 == *(short *)(iVar3 + 0x478c)) && (param_2[1] == *(short *)(iVar3 + 0x478e))) &&
        (param_2[2] == *(short *)(iVar3 + 0x4790))) && (*(ushort *)(iVar3 + 0x4792) >> 8 == param_1)
       ) {
      *(undefined2 *)(iVar3 + 0x4792) = 0xffff;
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 0x20);
  return;
}



/* ======================================================================
 * 00013af8  dup_cache_invalidate_exact
 * ====================================================================== */

void dup_cache_invalidate_exact(short *param_1,uint param_2)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  iVar1 = DAT_00013b38;
  uVar2 = 0;
  do {
    iVar3 = uVar2 * 0xc + iVar1;
    if ((((*param_1 == *(short *)(iVar3 + 0x478c)) && (param_1[1] == *(short *)(iVar3 + 0x478e))) &&
        (param_1[2] == *(short *)(iVar3 + 0x4790))) && (*(ushort *)(iVar3 + 0x4792) == param_2)) {
      *(undefined2 *)(iVar3 + 0x4792) = 0xffff;
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 0x20);
  return;
}



/* ======================================================================
 * 00013b44  lmc_post_event_200
 * ====================================================================== */

undefined4 lmc_post_event_200(undefined4 param_1)

{
  *(undefined4 *)(DAT_00013d68 + 0x10) = param_1;
  evt_flags_set(DAT_00013d6c,0x200);
  return 0;
}



/* ======================================================================
 * 00013b58  task_13b58
 * ====================================================================== */

void task_13b58(void)

{
  bool bVar1;
  char *pcVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  
  iVar3 = DAT_00013d78;
  iVar6 = DAT_00013d78 + DAT_00013d74 + 0x20;
  iVar5 = DAT_00013d78 + DAT_00013d74;
  if (*(char *)(DAT_00013d70 + 0xe) != '\0') {
    *(undefined1 *)(DAT_00013d70 + 0xe) = 0;
    *(undefined1 *)(iVar3 + 10) = *(undefined1 *)(iVar6 + 0x17);
    wsm_h_0B_join(*(undefined4 *)(iVar5 + 0x3c));
    return;
  }
  if (*(char *)(DAT_00013d70 + 0xf) != '\0') {
    *(undefined1 *)(DAT_00013d70 + 0xf) = 0;
    *(undefined1 *)(iVar3 + 10) = *(undefined1 *)(iVar6 + 0x16);
    wsm_h_17_start(*(undefined4 *)(iVar5 + 0x38));
    return;
  }
  bVar1 = false;
  iVar3 = *(int *)(DAT_00013d68 + 0x10);
  uVar8 = (uint)*(byte *)(DAT_00013d78 + 10);
  if (uVar8 == 2) {
    *DAT_00013d7c = 0;
  }
  iVar5 = 0;
  uVar7 = 0;
  do {
    iVar6 = uVar7 * 0x3b0 + DAT_00013d78;
    if (*(char *)(iVar6 + 0x19) != '\0') {
      if ((uVar7 == uVar8) || (iVar3 << 0x1e < 0)) {
        *(undefined2 *)(iVar6 + 0x2c) = 0;
        *(undefined2 *)(iVar6 + 0x2e) = 0;
        iVar4 = tx_abort_frames_for_vif();
        if (iVar4 == 0) {
          if ((*(char *)(iVar6 + 0x18) == '\x04') || (*(char *)(iVar6 + 0x18) == '\x06')) {
            timer_cancel(DAT_00013d80);
          }
          vif_teardown(uVar7 & 0xff);
        }
        else {
          bVar1 = true;
        }
      }
      else {
        iVar5 = iVar5 + 1;
      }
    }
    uVar7 = uVar7 + 1;
  } while (uVar7 < 3);
  iVar3 = DAT_00013d70 + -0x2c;
  if (bVar1) {
LAB_00013c38:
    evt_flags_set(DAT_00013d6c,0x200000);
    evt_flags_set(DAT_00013d6c,0x100000);
    timer_start(iVar3,0x100);
  }
  else {
    if (iVar5 == 0) {
      syn_scan_stop();
      iVar5 = tx_pending_count();
      pcVar2 = DAT_00013d84;
      if (iVar5 != 0) goto LAB_00013c38;
      if (*DAT_00013d84 == '\0') {
        mac_radio_stop_wrapper();
        *pcVar2 = '\x01';
        evt_flags_set(DAT_00013d6c,0x200);
        return;
      }
      sched_clear_pending();
      measure_ctl_reset(1);
      join_timeout();
      vif_cancel_join_timers(0);
      vif_cancel_join_timers(1);
      *DAT_00013d88 = 0;
      bab_free_sessions_for_vif(uVar8);
      lmc_flush_pending_tx();
      func_0xfff01094();
      measure_ctl_reset(0);
      *pcVar2 = '\0';
    }
    else if (uVar8 < 2) {
      bab_free_sessions_for_vif(uVar8);
    }
    hif_send_confirm_status8(0);
  }
  return;
}



/* ======================================================================
 * 00013cae  wsm_h_09_configuration_impl
 * ====================================================================== */

void wsm_h_09_configuration_impl(undefined4 *param_1)

{
  int iVar1;
  uint uVar2;
  uint uVar3;
  uint uVar4;
  int iVar5;
  short asStack_24 [2];
  short asStack_20 [2];
  short asStack_1c [2];
  short asStack_18 [2];
  
  iVar1 = DAT_00013d78;
  uVar2 = 0;
  do {
    iVar5 = uVar2 * 0x3b0 + iVar1;
    *(undefined4 *)(iVar5 + 0x11c) = *param_1;
    *(undefined4 *)(iVar5 + 0x120) = param_1[1];
    if (param_1[2] == 0) {
      *(undefined4 *)(iVar5 + 0x124) = 300;
    }
    else {
      *(undefined4 *)(iVar5 + 0x124) = param_1[2];
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 2);
  dbg_stats_config_apply(param_1 + 4);
  iVar1 = DAT_00013d8c;
  uVar2 = *(uint *)(DAT_00013d8c + 0x3c);
  *(undefined4 *)(DAT_00013d8c + 0x3c) = 0;
  if (*(short *)(param_1 + 3) != 0xc) {
    tlv_dispatch_default(param_1 + 6);
  }
  iVar5 = DAT_00013d90;
  if ((*(uint *)(iVar1 + 0x3c) & 1) != 0) {
    uVar2 = *(uint *)(iVar1 + 0x3c);
  }
  *param_1 = 0;
  uVar3 = 0;
  do {
    uVar4 = uVar3 + 1;
    *(undefined1 *)((int)param_1 + uVar3 + 4) = *(undefined1 *)(iVar5 + uVar3);
    uVar3 = uVar4;
  } while (uVar4 < 6);
  *(undefined1 *)((int)param_1 + 10) = *(undefined1 *)(iVar5 + 6);
  iVar1 = DAT_00013d94;
  *(undefined1 *)((int)param_1 + 0xb) = 0;
  param_1[3] = *(undefined4 *)(iVar1 + 0x14);
  phy_get_tx_power_range(asStack_18,asStack_1c,asStack_20,asStack_24);
  param_1[4] = (int)asStack_18[0];
  param_1[5] = (int)asStack_1c[0];
  param_1[7] = (int)asStack_20[0];
  param_1[6] = 0;
  param_1[8] = (int)asStack_24[0];
  param_1[9] = 0;
  phy_temp_compensate_all_slots();
  if ((uVar2 & 3) >> 1 == 0) {
    *param_1 = 2;
  }
  return;
}



/* ======================================================================
 * 00013d98  syn_scan_begin_request
 * ====================================================================== */

undefined4 syn_scan_begin_request(void *param_1)

{
  ushort *puVar1;
  sbyte sVar2;
  sbyte sVar3;
  byte bVar4;
  bool bVar5;
  int iVar6;
  sbyte *psVar7;
  uint *puVar8;
  undefined2 *puVar9;
  uint uVar10;
  sbyte *psVar11;
  uint uVar12;
  int iVar13;
  uint uVar14;
  uint uVar15;
  
  iVar6 = DAT_00014174;
  uVar10 = 0;
  if (*(char *)(DAT_00014174 + 0x14) != '\0') {
    return 4;
  }
  *(char *)(DAT_00014178 + 0x19) = (char)((uint)*(undefined4 *)((int)param_1 + 4) >> 0x18);
  *(uint *)((int)param_1 + 4) = *(uint *)((int)param_1 + 4) & 0xffffff;
  if ((0x22 < *(byte *)((int)param_1 + 9)) || (0x10 < *(byte *)((int)param_1 + 10))) {
    return 2;
  }
  for (uVar12 = 0; uVar12 < *(byte *)((int)param_1 + 9); uVar12 = uVar12 + 1) {
    uVar14 = *(uint *)((int)param_1 + uVar12 * 0x10 + 0x14);
    if (uVar14 == 0) {
      uVar10 = uVar10 | 1;
    }
    uVar15 = *(uint *)((int)param_1 + uVar12 * 0x10 + 0x10);
    if (uVar15 * 0x400 < (uint)*(byte *)((int)param_1 + 0xb)) {
      uVar10 = uVar10 | 2;
    }
    if (uVar14 < uVar15) {
      uVar10 = uVar10 | 4;
    }
  }
  if (uVar10 != 0) {
    return 2;
  }
  if (*(char *)(iVar6 + 0x14) != '\0') {
    return 4;
  }
  fw_memcpy(DAT_00014180,param_1,DAT_0001417c);
  psVar7 = DAT_00014180;
  if ((int)((uint)*(byte *)(DAT_00014184 + 0x1c) << 0x1d) < 0) {
    DAT_00014180[2] = DAT_00014180[2] | 2;
  }
  puVar8 = DAT_00014188;
  if (0x15 < (byte)psVar7[3]) {
    return 2;
  }
  if ((int)(*DAT_00014188 << 0x17) < 0) {
    return 5;
  }
  sVar2 = psVar7[1];
  uVar10 = *DAT_00014188 & 0xffffffef;
  *DAT_00014188 = uVar10;
  if ((sVar2 == 2) || (sVar2 == 3)) {
    *puVar8 = uVar10 | 0x10;
  }
  bVar5 = false;
  uVar10 = 0;
  do {
    iVar13 = uVar10 * 0x3b0 + DAT_0001418c;
    uVar12 = *(uint *)(iVar13 + 0x1c);
    if ((uVar12 & 0x3f) >> 2 == 0) {
      if ((uVar12 & 1) != 0) {
        if (*(char *)(uVar10 * 0x104 + DAT_00014190 + 0x59) == '\0') {
          return 8;
        }
        if (-1 < (int)(uVar12 << 0x12)) {
          *(uint *)(iVar13 + 0x1c) = uVar12 | 0x2000;
          lmc_recompute_vif_roles();
        }
        goto LAB_00013ea0;
      }
      if ((int)(uVar12 << 0x19) < 0) {
        return 8;
      }
    }
    else {
LAB_00013ea0:
      bVar5 = true;
    }
    psVar7 = DAT_00014180;
    uVar10 = uVar10 + 1;
    if (2 < uVar10) {
      if (bVar5 && !bVar5) {
        return 8;
      }
      DAT_00014180[1] = bVar5;
      if (psVar7[10] == 0) {
        psVar11 = (sbyte *)0x0;
      }
      else {
        psVar11 = psVar7 + (uint)(byte)psVar7[9] * 0x10 + 0xc;
      }
      *(sbyte **)(DAT_00014174 + -0x6c) = psVar11;
      *puVar8 = *puVar8 & DAT_00014194;
      timer_cancel(DAT_00014184 + -0x34);
      vif_get_bssid(DAT_00014198,((byte)psVar7[2] & 0x1f) >> 4);
      iVar13 = DAT_0001419c;
      puVar9 = DAT_00014198;
      *(undefined2 *)(DAT_0001419c + 0xc) = *DAT_00014198;
      *(undefined2 *)(iVar13 + 0xe) = puVar9[1];
      *(undefined2 *)(iVar13 + 0x10) = puVar9[2];
      iVar13 = DAT_000141a0;
      puVar1 = (ushort *)(DAT_000141a0 + 2);
      *(ushort *)(DAT_000141a0 + 0x42) = *puVar1;
      fw_memcpy(*(void **)(iVar13 + 0x44),*(void **)(iVar13 + 4),(uint)*puVar1);
      sVar3 = psVar7[1];
      if (((sVar3 == 1) && ((int)(*puVar8 << 0x1b) < 0)) && (sVar2 == 3)) {
        *psVar7 = (sbyte)((*(uint *)(psVar7 + 0xc) & 0x3fff) >> 0xd);
      }
      *(char *)(DAT_00014184 + 0x2d) = (char)(1 << *psVar7);
      if (sVar3 == 1) {
        if (((int)(*puVar8 << 0x1b) < 0) && (sVar2 == 3)) {
          *(undefined1 *)((uint)*(byte *)(DAT_00014178 + -0xa9) * 0x3b0 + DAT_0001418c + 0x388) = 0;
          *(undefined1 *)(DAT_00014178 + -0x9c) = 1;
          return 0;
        }
        bVar4 = psVar7[2];
        if ((bVar4 & 1) != 0) {
          *puVar8 = *puVar8 | 0x1000;
        }
        if ((int)((uint)bVar4 << 0x1e) < 0) {
          *puVar8 = *puVar8 | 0x2000;
        }
      }
      puVar8 = DAT_000141a4;
      *(undefined1 *)(iVar6 + 0x14) = 1;
      evt_flags_set(puVar8,0x400);
      return 0;
    }
  } while( true );
}



/* ======================================================================
 * 00013fa0  syn_scan_set_state
 * ====================================================================== */

void syn_scan_set_state(undefined1 param_1,int param_2)

{
  *(undefined1 *)(DAT_00014174 + 0x14) = param_1;
  if (param_2 != 0) {
    timer_start(DAT_00014184 + -0x34);
    return;
  }
  evt_flags_set(DAT_000141a4,0x400);
  return;
}



/* ======================================================================
 * 00013fc0  syn_scan_restore_channel
 * ====================================================================== */

void syn_scan_restore_channel
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined2 *puVar1;
  int iVar2;
  undefined4 local_10;
  undefined4 uStack_c;
  undefined4 uStack_8;
  
  local_10 = param_2;
  uStack_c = param_3;
  uStack_8 = param_4;
  mac_reset_and_drain();
  iVar2 = DAT_00014198;
  *(undefined1 *)(DAT_00014198 + -0x1b) = 0;
  *(undefined4 *)(iVar2 + -0x18) = 0;
  *(undefined2 *)(iVar2 + 0xe) = 0;
  local_10 = CONCAT31(local_10._1_3_,3);
  mac_apply_channel_and_vif_config(&local_10);
  iVar2 = DAT_0001419c;
  puVar1 = DAT_0001418c;
  DAT_0001418c[1] = *(undefined2 *)(DAT_0001419c + 0x9c);
  *puVar1 = *(undefined2 *)(iVar2 + 0x9a);
  *(undefined1 *)(puVar1 + 2) = *(undefined1 *)(iVar2 + 0x99);
  evt_flags_set(DAT_000141a4,0x200000);
  return;
}



/* ======================================================================
 * 00014000  syn_scan_finish_and_confirm
 * ====================================================================== */

void syn_scan_finish_and_confirm
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,uint param_4)

{
  int iVar1;
  int iVar2;
  uint *puVar3;
  undefined1 uVar4;
  ushort uVar5;
  uint uVar6;
  int iVar7;
  undefined4 local_1c;
  undefined4 local_18;
  
  *(undefined1 *)(DAT_00014184 + 0x2d) = 0;
  iVar2 = DAT_00014178;
  iVar1 = DAT_00014174;
  *(undefined1 *)(DAT_00014174 + 0x14) = 0;
  *(undefined2 *)(iVar2 + -0xac) = 0;
  iVar2 = DAT_00014198;
  puVar3 = DAT_00014188;
  *(undefined1 *)(DAT_00014198 + -0x1b) = 0;
  *(undefined4 *)(iVar2 + -0x18) = 0;
  *(undefined2 *)(iVar2 + 0xe) = 0;
  *puVar3 = *puVar3 & DAT_000141a8;
  uVar6 = 0;
  do {
    local_1c = param_1;
    if (*(char *)(uVar6 * 0x3b0 + DAT_0001418c + 0x19) != '\0') {
      if (uVar6 < 3) {
        local_18 = param_4;
        syn_scan_restore_channel();
        uVar4 = ps_exit_sleep_and_resume();
        local_18 = CONCAT31(local_18._1_3_,uVar4);
        evt_flags_set(DAT_000141a4,0x200000);
        goto LAB_0001406e;
      }
      break;
    }
    uVar6 = uVar6 + 1;
  } while (uVar6 < 3);
  local_18 = param_4 & 0xffffff00;
  *(undefined2 *)(DAT_0001418c + 2) = 0;
  mac_radio_stop();
LAB_0001406e:
  iVar7 = measure_is_state5();
  iVar2 = DAT_00014178;
  if (iVar7 != 0) {
    phy_maybe_notify();
    goto LAB_000140ba;
  }
  if ((int)(*puVar3 << 0x1b) < 0) {
    syn_scan_set_state(1,*(int *)(DAT_00014180 + 4) << 10);
    if (*(char *)(iVar2 + -0x9b) == '\0') goto LAB_000140ba;
    local_18._0_2_ = CONCAT11(*(undefined1 *)(iVar1 + 0x15),(undefined1)local_18);
LAB_000140ae:
    uVar5 = (ushort)*(byte *)(iVar2 + -0x9a);
  }
  else {
    local_18._0_2_ = CONCAT11(*(undefined1 *)(iVar1 + 0x15),(undefined1)local_18);
    uVar5 = 0;
    if (*(char *)(DAT_00014178 + -0x99) != '\0') goto LAB_000140ae;
  }
  local_18 = CONCAT22(uVar5,(undefined2)local_18);
  ind_0806_scan_complete(&local_1c);
LAB_000140ba:
  *(undefined4 *)(iVar2 + -0x98) = 0;
  return;
}



/* ======================================================================
 * 000140be  syn_scan_dwell_next
 * ====================================================================== */

void syn_scan_dwell_next(void)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined1 uVar4;
  
  iVar1 = *(int *)((uint)*(byte *)(DAT_00014174 + 0x15) * 0x10 + DAT_00014180 + 0x14);
  iVar3 = *(int *)(DAT_00014184 + -0x7c);
  iVar2 = *DAT_00014188;
  if ((int)((uint)*(byte *)(DAT_00014184 + 0x1c) << 0x1d) < 0) {
    uVar4 = 3;
  }
  else {
    if ((*(byte *)(DAT_00014184 + 0x1c) & 1) == 0) {
      *(undefined1 *)(DAT_00014174 + 0x14) = 6;
      evt_flags_set(DAT_000141a4,0x400);
      return;
    }
    uVar4 = 2;
  }
  *(undefined1 *)(DAT_00014174 + 0x14) = uVar4;
  ps_check_pending_wake(-1 < iVar2 << 0x13,iVar3 * 2 + iVar1 * 0x400);
  return;
}



/* ======================================================================
 * 00014110  syn_scan_set_band_and_program
 * ====================================================================== */

void syn_scan_set_band_and_program(int param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  ushort *puVar1;
  ushort uVar2;
  int iVar3;
  int local_18;
  undefined4 local_14;
  undefined4 uStack_10;
  
  puVar1 = DAT_0001418c;
  if (param_1 == 1) {
    iVar3 = (uint)*DAT_0001418c << 0x1b;
  }
  else {
    if (param_1 != 0) {
      return;
    }
    iVar3 = (uint)*DAT_0001418c << 0x1a;
  }
  if (iVar3 < 0) {
    local_18 = param_2;
    local_14 = param_3;
    uStack_10 = param_4;
    uVar2 = phy_build_rate_cfg(param_1,DAT_000141ac,0,DAT_0001418c[1]);
    iVar3 = DAT_00014180;
    *puVar1 = uVar2;
    *(char *)(puVar1 + 2) = (char)((*(byte *)(iVar3 + 2) & 7) >> 2);
    phy_set_band_reg();
    iVar3 = DAT_00014198;
    *(undefined1 *)(DAT_00014198 + -0x1b) = 2;
    *(undefined4 *)(iVar3 + -0x18) = 0x4000;
    local_18 = (uint)CONCAT21(*puVar1,(char)puVar1[2]) << 8;
    local_14 = CONCAT13(2,CONCAT12(1,puVar1[1]));
    mac_apply_channel_and_vif_config(&local_18);
  }
  return;
}



/* ======================================================================
 * 000141b0  syn_scan_build_probe_req
 * ====================================================================== */

undefined4 syn_scan_build_probe_req(uint param_1)

{
  byte *pbVar1;
  byte bVar2;
  ushort *puVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  int iVar7;
  uint uVar8;
  void *src;
  int iVar9;
  char *src_00;
  ushort *puVar10;
  undefined1 *dst;
  void *dst_00;
  ushort local_24;
  
  iVar5 = tx_ctx_alloc_init(6,0,1);
  iVar4 = DAT_000145c4;
  puVar3 = DAT_000145c0;
  if (iVar5 == 0) {
    return 0;
  }
  dst_00 = *(void **)(iVar5 + 0x1c);
  iVar6 = *(int *)(DAT_000145c0 + param_1 * 0x1d8 + 0xe);
  puVar10 = (ushort *)0x0;
  iVar9 = 0;
  if (iVar6 << 0x11 < 0) {
    puVar10 = (ushort *)
              ((uint)*(byte *)(DAT_000145c4 + 0x17) * 0x24 +
              (uint)*(byte *)(DAT_000145c8 + 9) * 0x10 + DAT_000145c8 + 0xc);
    iVar7 = 1;
    if (*(char *)(DAT_000145c8 + 10) != '\0') {
      iVar9 = *(int *)puVar10;
    }
  }
  else {
    iVar7 = 0;
    if (param_1 < 2) {
      puVar10 = DAT_000145c0 + param_1 * 0x1d8 + 0x76;
      iVar9 = *(int *)puVar10;
    }
  }
  iVar7 = iVar7 * 0x40 + DAT_000145cc;
  if (*(short *)(iVar7 + 2) == 0) {
    fw_assert(s_syn_scan_c_000145d0,0x35c,0x2e);
  }
  src = *(void **)(iVar7 + 4);
  local_24 = *(ushort *)(iVar7 + 2);
  uVar8 = (uint)local_24;
  fw_memcpy(dst_00,src,uVar8);
  iVar7 = *(int *)(iVar5 + 0x1c);
  *(ushort *)(iVar7 + 10) = puVar3[param_1 * 0x1d8 + 0x1a];
  *(ushort *)(iVar7 + 0xc) = puVar3[param_1 * 0x1d8 + 0x1b];
  *(ushort *)(iVar7 + 0xe) = puVar3[param_1 * 0x1d8 + 0x1c];
  *(undefined1 *)((int)dst_00 + 0x18) = 0;
  if (iVar9 == 0) {
    dst = (undefined1 *)((int)dst_00 + 0x1a);
    *(undefined1 *)((int)dst_00 + 0x19) = 0;
  }
  else {
    *(char *)((int)dst_00 + 0x19) = (char)iVar9;
    fw_memcpy((void *)((int)dst_00 + 0x1a),puVar10 + 2,iVar9);
    dst = (undefined1 *)((int)dst_00 + 0x1a + iVar9);
    local_24 = local_24 + (short)iVar9;
  }
  src_00 = (char *)((int)src + 0x1a);
  iVar9 = ((int)((uVar8 - 0x18) * 0x10000) >> 0x10) + -2;
  do {
    iVar9 = (int)(short)iVar9;
    while( true ) {
      if (iVar9 < 1) {
        *(ushort *)(iVar5 + 0x5c) = local_24;
        *(undefined1 *)(iVar5 + 0xbf) = 0xf;
        *(char *)(iVar5 + 0xbd) = (char)param_1;
        iVar9 = DAT_000145c8;
        if (iVar6 << 0x11 < 0) {
          bVar2 = *(byte *)(DAT_000145c8 + 3);
          *(byte *)(iVar5 + 0xc) = bVar2;
          if (((int)((uint)*DAT_000145c0 << 0x1a) < 0) && (bVar2 < 6)) {
            *(undefined1 *)(iVar5 + 0xc) = 6;
          }
          *(undefined4 *)(iVar5 + 0x98) =
               *(undefined4 *)((uint)*(byte *)(iVar4 + 0x15) * 0x10 + iVar9 + 0x18);
        }
        else {
          *(undefined1 *)(iVar5 + 0xc) = 0xff;
        }
        lmc_tx_assign_default_rate(iVar5);
        return 1;
      }
      if (*src_00 == '\x03') break;
      uVar8 = (uint)(byte)src_00[1];
      fw_memcpy(dst,src_00,uVar8 + 2);
      src_00 = src_00 + uVar8 + 2;
      iVar9 = (int)(((iVar9 - uVar8) + -2) * 0x10000) >> 0x10;
      dst = dst + uVar8 + 2;
    }
    *dst = 3;
    dst[1] = 1;
    dst[2] = (char)DAT_000145c0[1];
    pbVar1 = (byte *)(src_00 + 1);
    dst = dst + 3;
    src_00 = src_00 + *pbVar1 + 2;
    iVar9 = iVar9 - (*pbVar1 + 2);
  } while( true );
}



/* ======================================================================
 * 00014332  syn_scan_maybe_send_probe
 * ====================================================================== */

undefined4 syn_scan_maybe_send_probe(int param_1)

{
  int iVar1;
  
  if (((*(uint *)(param_1 * 0x3b0 + DAT_000145c0 + 0x1c) & 0x4400) != 0) &&
     (iVar1 = syn_scan_build_probe_req(), iVar1 != 0)) {
    return 1;
  }
  return 0;
}



/* ======================================================================
 * 00014358  syn_scan_program_channel
 * ====================================================================== */

void syn_scan_program_channel(undefined4 param_1,int param_2,undefined4 param_3,undefined4 param_4)

{
  undefined2 *puVar1;
  undefined1 *puVar2;
  int iVar3;
  undefined2 uVar4;
  int local_18;
  undefined4 local_14;
  undefined4 uStack_10;
  
  puVar2 = DAT_000145c8;
  puVar1 = DAT_000145c0;
  local_18 = param_2;
  local_14 = param_3;
  uStack_10 = param_4;
  uVar4 = phy_build_rate_cfg(*DAT_000145c8,DAT_000145dc,0,DAT_000145c0[1]);
  *puVar1 = uVar4;
  *(char *)(puVar1 + 2) = (char)(((byte)puVar2[2] & 7) >> 2);
  phy_set_band_reg();
  iVar3 = DAT_000145e0;
  *(undefined1 *)(DAT_000145e0 + 0x19) = 2;
  *(undefined4 *)(iVar3 + 0x1c) = 0x4000;
  local_18 = (uint)CONCAT21(*puVar1,*(undefined1 *)(puVar1 + 2)) << 8;
  local_14 = CONCAT13(2,CONCAT12(1,puVar1[1]));
  mac_apply_channel_and_vif_config(&local_18);
  return;
}



/* ======================================================================
 * 000143a6  task_143a6
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x000143c6) */
/* WARNING: Removing unreachable block (ram,0x000143c6) */

void task_143a6(void)

{
  uint uVar1;
  
  uVar1 = (uint)*(byte *)(DAT_000145c4 + 0x14);
                    /* WARNING: Could not recover jumptable at 0x000143c6. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_000143ca <= uVar1) {
    uVar1 = (uint)DAT_000143ca;
  }
  (*(code *)((uint)*(byte *)(uVar1 + 0x143cb) * 2 + 0x143cb))(*DAT_000145ec);
  return;
}



/* ======================================================================
 * 0001475a  syn_scan_complete_if_scanning
 * ====================================================================== */

void syn_scan_complete_if_scanning(undefined2 param_1)

{
  int iVar1;
  
  iVar1 = DAT_000148a4;
  if (*(char *)(DAT_000148a4 + 0x14) == '\x02') {
    *(undefined2 *)(DAT_000148a4 + 0x18) = param_1;
    *(undefined1 *)(iVar1 + 0x14) = 6;
    evt_flags_set(DAT_000148a0,0x400);
  }
  return;
}



/* ======================================================================
 * 00014774  syn_scan_abort
 * ====================================================================== */

void syn_scan_abort(undefined2 param_1)

{
  char cVar1;
  int iVar2;
  
  iVar2 = DAT_000148a4;
  *(undefined2 *)(DAT_000148a4 + 0x18) = param_1;
  cVar1 = *(char *)(iVar2 + 0x14);
  if ((cVar1 == '\x02') || (cVar1 == '\x03')) {
    *(undefined1 *)(iVar2 + 0x14) = 6;
    evt_flags_set(DAT_000148a0,0x400);
  }
  return;
}



/* ======================================================================
 * 00014794  syn_scan_stop
 * ====================================================================== */

void syn_scan_stop(void)

{
  if (*(char *)(DAT_000148a4 + 0x14) != '\0') {
    timer_cancel(DAT_0001488c + 0x6c);
    if (*(char *)(DAT_000148a8 + 1) == '\x01') {
      syn_scan_restore_channel();
    }
    syn_scan_finish_and_confirm(0);
  }
  return;
}



/* ======================================================================
 * 000147ba  syn_scan_stop_dup
 * ====================================================================== */

void syn_scan_stop_dup(void)

{
  if (*(char *)(DAT_000148a4 + 0x14) != '\0') {
    if (*(char *)(DAT_000148a4 + 0x14) != '\0') {
      timer_cancel(DAT_0001488c + 0x6c);
      if (*(char *)(DAT_000148a8 + 1) == '\x01') {
        syn_scan_restore_channel();
      }
      syn_scan_finish_and_confirm(0);
    }
    return;
  }
  return;
}



/* ======================================================================
 * 000147c6  syn_scan_probe_tx_done
 * ====================================================================== */

void syn_scan_probe_tx_done(int param_1)

{
  *(byte *)(DAT_0001488c + 10) = *(byte *)(DAT_0001488c + 10) & 0xfd;
  if (*(char *)((uint)*(byte *)(param_1 + 0xbd) * 0x104 + DAT_000148ac + 0x40) == '\0') {
    mac_set_ps_bit();
  }
  if (*(char *)(DAT_000148a4 + 0x14) == '\x10') {
    *(undefined2 *)(DAT_000148a4 + 0x18) = *(undefined2 *)(param_1 + 0x70);
    evt_flags_set(DAT_000148a0,0x400);
  }
  return;
}



/* ======================================================================
 * 00014806  syn_scan_stop_clear_flag
 * ====================================================================== */

void syn_scan_stop_clear_flag(void)

{
  *DAT_00014894 = *DAT_00014894 & 0xffffffef;
  if (*(char *)(DAT_000148a4 + 0x14) != '\0') {
    if (*(char *)(DAT_000148a4 + 0x14) != '\0') {
      timer_cancel(DAT_0001488c + 0x6c);
      if (*(char *)(DAT_000148a8 + 1) == '\x01') {
        syn_scan_restore_channel();
      }
      syn_scan_finish_and_confirm(0);
    }
    return;
  }
  return;
}



/* ======================================================================
 * 0001481c  syn_scan_try_start
 * ====================================================================== */

void syn_scan_try_start(void)

{
  uint *puVar1;
  
  puVar1 = DAT_00014894;
  if (((*DAT_000148b0 != '\0') && (DAT_000148b0[1] == '\0')) && (-1 < (int)(*DAT_00014894 << 0x1a)))
  {
    DAT_000148b0[1] = '\x01';
    *puVar1 = *puVar1 & 0xffffdfff | 0x1000;
    *(undefined1 *)(DAT_000148a4 + 0x14) = 1;
    evt_flags_set(DAT_000148a0,0x400);
  }
  return;
}



/* ======================================================================
 * 00014852  syn_scan_filter_rx_frame
 * ====================================================================== */

undefined4 syn_scan_filter_rx_frame(int param_1)

{
  undefined4 uVar1;
  
  uVar1 = 1;
  if (*(char *)(DAT_000148b0 + 1) != '\0') {
    if ((int)(*(uint *)(param_1 + 0x20) << 0x14) < 0) {
      if (*(short *)(param_1 + 0x12) != 0x50) {
        if (*(short *)(param_1 + 0x12) != 0x80) {
          return 1;
        }
        *(uint *)(param_1 + 0x20) = *(uint *)(param_1 + 0x20) | 0x80000000;
        return 1;
      }
    }
    else if (*(byte *)(DAT_000148b0 + 2) + 5 <= (uint)*(byte *)(param_1 + 0x15)) {
      return 1;
    }
    uVar1 = 0;
  }
  return uVar1;
}



/* ======================================================================
 * 000148b4  ie_rates_have_non_cck
 * ====================================================================== */

undefined4 ie_rates_have_non_cck(int param_1)

{
  byte bVar1;
  byte *pbVar2;
  byte bVar3;
  
  pbVar2 = (byte *)(param_1 + 2);
  bVar3 = 0;
  while( true ) {
    if (*(byte *)(param_1 + 1) <= bVar3) {
      return 0;
    }
    bVar1 = *pbVar2 & 0x7f;
    if ((((bVar1 != 2) && (bVar1 != 4)) && (bVar1 != 0xb)) && (bVar1 != 0x16)) break;
    pbVar2 = pbVar2 + 1;
    bVar3 = bVar3 + 1;
  }
  return 1;
}



/* ======================================================================
 * 000148ea  ie_peer_supports_ofdm
 * ====================================================================== */

undefined4 ie_peer_supports_ofdm(byte *param_1,int param_2)

{
  byte *pbVar1;
  int iVar2;
  undefined4 uVar3;
  
  uVar3 = 0;
  pbVar1 = ie_find(param_1,param_2,1,0);
  if (((pbVar1 != (byte *)0x0) && (iVar2 = ie_rates_have_non_cck(), iVar2 != 0)) ||
     ((pbVar1 = ie_find(param_1,param_2,0x32,0), pbVar1 != (byte *)0x0 &&
      (iVar2 = ie_rates_have_non_cck(), iVar2 != 0)))) {
    uVar3 = 1;
  }
  return uVar3;
}



/* ======================================================================
 * 00014924  syn_start_load_beacon_template
 * ====================================================================== */

/* syn_start_load_beacon_template(if_id, offset) -- copy the host-supplied beacon
   template into the live beacon buffer and derive capability flags from it.
   
   *** THIS IS THE RECORDED AP-MODE CRASH SITE. ***
     if (*(u16 *)(g_templates + if_id*0x40 + 10) == 0)
         fw_assert("syn_start.c", 1147, 0x2F);
   
   The length field at template+10 is zero until the host writes MIB 0x1002
   TEMPLATE_FRAME with type 1 (BEACON).  Issuing WSM_START (0x0017) before that
   template write reaches this assert, which is a hard stop -- the firmware builds
   the 0x0800 exception indication and spins forever
   (exc_build_indication_and_spin), so the device is dead until reload.
   
   That matches the crash in PROGRESS.md exactly: PC 0x0001493F with the Thumb bit,
   i.e. 0x0001493E inside this function.  symbolize.py resolves it, and
   assert_table.py gives the (file, line, code) triple directly.
   
   **Driver rule: write the beacon template MIB before WSM_START, always.** There is
   no soft-failure path and no confirm status that reports it.
   
   What the function does once the length is valid:
   
     src = template->data;  len = template->len;
     parse the SSID IE (id 0) in the body at src+0x24:
         len 0 or (len 1 and ssid[0] == 0)  ->  set vif flag 0x2000000 (hidden SSID)
         otherwise                          ->  clear it
     if (!ie_peer_supports_ofdm(body))      ->  set vif flag 0x10000 (CCK-only BSS)
     copy the whole template into the per-vif beacon buffer at
         g_beacon + if_id*0x70 + 0x10, recording len at +0x18
     set the TX descriptor fields: +0x1A = 0x80, +0x2C = 0xFE (pending),
         +0x1F = template flags & 0x7F, +0x14 = 0x300
   
   So the hidden-SSID and CCK-only decisions for AP mode are taken from the beacon
   template itself, not from any WSM field -- a driver that writes a template with a
   zero-length SSID gets hidden-SSID behaviour implicitly. */

void syn_start_load_beacon_template(int param_1,int param_2)

{
  ushort uVar1;
  void *src;
  byte *buf;
  byte *pbVar2;
  uint uVar3;
  void *dst;
  int iVar4;
  int iVar5;
  int iVar6;
  int iVar7;
  
  iVar6 = param_1 * 0x40 + DAT_00014d20;
  if (*(short *)(iVar6 + 10) == 0) {
    fw_assert(s_syn_start_c_00014d28,DAT_00014d24,0x2f);
  }
  src = *(void **)(iVar6 + 0xc);
  uVar1 = *(ushort *)(iVar6 + 10);
  iVar5 = param_1 * 0x3b0 + DAT_00014d34;
  iVar4 = uVar1 - 0x24;
  buf = (byte *)((int)src + 0x24);
  pbVar2 = ie_find(buf,iVar4,0,0);
  if (pbVar2 == (byte *)0x0) goto LAB_00014990;
  if (pbVar2[1] == 1) {
    if (pbVar2[2] != 0) goto LAB_0001498a;
LAB_00014984:
    uVar3 = *(uint *)(iVar5 + 0x1c) | 0x2000000;
  }
  else {
    if (pbVar2[1] == 0) goto LAB_00014984;
LAB_0001498a:
    uVar3 = *(uint *)(iVar5 + 0x1c) & 0xfdffffff;
  }
  *(uint *)(iVar5 + 0x1c) = uVar3;
LAB_00014990:
  iVar4 = ie_peer_supports_ofdm(buf,iVar4);
  if (iVar4 == 0) {
    *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) | 0x10000;
  }
  iVar4 = DAT_00014d34;
  iVar7 = param_1 * 0x70 + DAT_00014d38;
  *(char *)(iVar7 + 0x79) = (char)param_1;
  *(byte *)(iVar7 + 0x7a) = *(byte *)(iVar5 + 0x1b) & 1;
  dst = *(void **)(iVar7 + 0x10);
  *(int *)(iVar4 + 0x10) = (int)dst + param_2;
  fw_memcpy(dst,src,(uint)uVar1);
  *(ushort *)(iVar7 + 0x18) = uVar1;
  *(undefined2 *)(iVar7 + 0x1a) = 0x80;
  *(undefined2 *)(iVar7 + 0x2e) = 0;
  *(undefined2 *)(iVar7 + 0x2c) = 0xfe;
  *(byte *)(iVar7 + 0x1f) = *(byte *)(iVar6 + 9) & 0x7f;
  if (-1 < (int)((uint)*(byte *)(iVar6 + 9) << 0x18)) {
    *(uint *)(iVar7 + 0x14) = *(uint *)(iVar7 + 0x14) | 8;
  }
  *(undefined4 *)(iVar7 + 0x14) = 0x300;
  return;
}



/* ======================================================================
 * 00014a00  lmc_recompute_vif_roles
 * ====================================================================== */

void lmc_recompute_vif_roles(void)

{
  int iVar1;
  byte bVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  uint uVar6;
  
  iVar1 = DAT_00014d3c;
  uVar3 = 0;
  *(undefined1 *)(DAT_00014d3c + 0x1c) = 0;
  do {
    iVar4 = uVar3 * 0x3b0 + DAT_00014d34;
    if (*(char *)(iVar4 + 0x19) != '\0') {
      *(undefined1 *)(iVar4 + 0x32) = 0;
      uVar6 = *(uint *)(iVar4 + 0x1c);
      bVar2 = 0;
      if ((uVar6 & 0x1f) >> 2 != 0) {
        if ((int)(uVar6 << 0x1c) < 0) {
          bVar2 = 1;
        }
        else {
          bVar2 = 4;
        }
      }
      if ((uVar6 & 1) != 0) {
        iVar5 = uVar3 * 0x104 + DAT_00014d40;
        if ((int)(uVar6 << 0x1a) < 0) {
          *(undefined1 *)(iVar5 + 0x41) = 0;
LAB_00014a82:
          bVar2 = bVar2 | 4;
        }
        else {
          if ((int)(uVar6 << 0x12) < 0) {
            *(undefined1 *)(iVar5 + 0x41) = 1;
          }
          else if (*(char *)(iVar5 + 0x41) == '\0') goto LAB_00014a82;
          iVar5 = (~uVar3 & 1) * 0x3b0 + DAT_00014d34;
          if ((*(char *)(iVar5 + 0x19) != '\0') &&
             (*(short *)(iVar5 + 0x42) == *(short *)(iVar4 + 0x42))) {
            bVar2 = bVar2 | 8;
          }
          bVar2 = bVar2 | 1;
        }
      }
      *(byte *)(iVar4 + 0x32) = bVar2;
      *(byte *)(iVar1 + 0x1c) = bVar2 | *(byte *)(iVar1 + 0x1c);
    }
    uVar3 = uVar3 + 1;
    if (2 < uVar3) {
      return;
    }
  } while( true );
}



/* ======================================================================
 * 00014a96  vif_enter_operating_state
 * ====================================================================== */

void vif_enter_operating_state(int param_1)

{
  char cVar1;
  int iVar2;
  int iVar3;
  
  lmc_recompute_vif_roles();
  iVar2 = param_1 * 0x3b0 + DAT_00014d34;
  *(short *)(iVar2 + 0x2c) = (short)DAT_00014d44;
  *(undefined2 *)(iVar2 + 0x15c) = 0;
  *(undefined2 *)(iVar2 + 0x15e) = 0;
  *(char *)(iVar2 + 0x51) = (char)param_1;
  *(undefined2 *)(iVar2 + 0x52) = *(undefined2 *)(iVar2 + 0x42);
  *(undefined4 *)(iVar2 + 0x54) = 0xffffffff;
  iVar3 = DAT_00014d38;
  *(undefined4 *)(iVar2 + 0x5c) = 0;
  iVar3 = param_1 * 0x98 + iVar3;
  *(undefined1 *)(iVar3 + 0x492) = 0;
  *(undefined1 *)(iVar3 + 0x493) = 0;
  cVar1 = *(char *)(iVar2 + 0x18);
  if (((cVar1 == '\x03') || (cVar1 == '\x04')) || (cVar1 == '\x06')) {
    *(undefined1 *)(iVar2 + 0x50) = 0x20;
  }
  iVar3 = DAT_00014d3c;
  *(undefined1 *)(iVar2 + 0x50) = 0x23;
  *(undefined1 *)(iVar3 + -0x5c) = 1;
  lmc_sched_request_radio(iVar2 + 0x44);
  if (param_1 == 2) {
    timer_start(iVar3 + 8,&DAT_0004d000);
  }
  if (*(char *)(iVar3 + -0x5a) == '\0') {
    *(undefined1 *)(iVar3 + -0x5a) = 1;
  }
  return;
}



/* ======================================================================
 * 00014b1a  syn_start_finish_vif
 * ====================================================================== */

void syn_start_finish_vif(void)

{
  int iVar1;
  uint uVar2;
  
  uVar2 = (uint)*(byte *)(DAT_00014d34 + 10);
  iVar1 = uVar2 * 0x3b0 + DAT_00014d34;
  vif_enter_operating_state(uVar2);
  if (*(int *)(iVar1 + 0x1c) << 0x1d < 0) {
    syn_start_load_beacon_template(uVar2,0);
    *(uint *)(iVar1 + 0x1c) = *(uint *)(iVar1 + 0x1c) & 0xfffbffff | 0x20000;
    beacon_schedule_next(uVar2);
  }
  if (*(int *)(iVar1 + 0x1c) << 0x1e < 0) {
    ps_resync_beacon_state(uVar2);
  }
  mac_set_state3();
  *DAT_00014d48 = 0xff;
  return;
}



/* ======================================================================
 * 00014b6a  vif_get_bssid
 * ====================================================================== */

void vif_get_bssid(undefined2 *param_1,int param_2)

{
  int iVar1;
  
  iVar1 = param_2 * 6 + DAT_00014d38;
  *param_1 = *(undefined2 *)(iVar1 + 0x454);
  param_1[1] = *(undefined2 *)(iVar1 + 0x456);
  param_1[2] = *(undefined2 *)(iVar1 + 0x458);
  return;
}



/* ======================================================================
 * 00014b86  syn_start_register_channel_use
 * ====================================================================== */

void syn_start_register_channel_use(int param_1)

{
  uint uVar1;
  short *psVar2;
  short *psVar3;
  
  uVar1 = 0;
  do {
    psVar2 = (short *)(uVar1 * 0xc + DAT_00014d34 + DAT_00014d4c);
    if (((char)psVar2[4] != '\0') && (*psVar2 == *(short *)(param_1 + 0x2a))) break;
    uVar1 = uVar1 + 1;
    psVar2 = (short *)0x0;
  } while (uVar1 < 3);
  if (psVar2 == (short *)0x0) {
    uVar1 = 0;
    do {
      psVar3 = (short *)(uVar1 * 0xc + DAT_00014d34 + DAT_00014d4c);
      if ((char)psVar3[4] == '\0') {
        *psVar3 = *(short *)(param_1 + 0x2a);
        psVar2 = psVar3;
        break;
      }
      uVar1 = uVar1 + 1;
    } while (uVar1 < 3);
  }
  if (2 < *(byte *)(psVar2 + 4)) {
    fw_assert(s_syn_start_c_00014d28,DAT_00014d54,DAT_00014d50);
  }
  *(undefined1 *)((int)psVar2 + *(byte *)(psVar2 + 4) + 9) = *(undefined1 *)(param_1 + 2);
  *(char *)(psVar2 + 4) = (char)psVar2[4] + '\x01';
  return;
}



/* ======================================================================
 * 00014bf4  syn_start_release_channel_use
 * ====================================================================== */

void syn_start_release_channel_use(int param_1)

{
  char cVar1;
  uint uVar2;
  uint uVar3;
  short *psVar4;
  
  uVar2 = 0;
  do {
    psVar4 = (short *)(uVar2 * 0xc + DAT_00014d34 + DAT_00014d4c);
    if (*psVar4 == *(short *)(param_1 + 0x2a)) break;
    uVar2 = uVar2 + 1;
    psVar4 = (short *)0x0;
  } while (uVar2 < 3);
  if (psVar4 == (short *)0x0) {
    fw_assert(s_syn_start_c_00014d28,DAT_00014d54 + 0x18,DAT_00014d50);
  }
  if ((char)psVar4[4] == '\0') {
    fw_assert(s_syn_start_c_00014d28,DAT_00014d54 + 0x19,DAT_00014d50 + 1);
  }
  uVar3 = (uint)*(byte *)(psVar4 + 4);
  uVar2 = 0;
  do {
    if (uVar3 <= uVar2) {
LAB_00014c66:
      cVar1 = (char)psVar4[4] + -1;
      *(char *)(psVar4 + 4) = cVar1;
      if (cVar1 == '\0') {
        *psVar4 = 0;
      }
      return;
    }
    if (*(char *)((int)psVar4 + uVar2 + 9) == *(char *)(param_1 + 2)) {
      if (uVar2 + 1 < uVar3) {
        *(undefined1 *)((int)psVar4 + uVar2 + 9) = *(undefined1 *)((int)psVar4 + uVar3 + 8);
      }
      goto LAB_00014c66;
    }
    uVar2 = uVar2 + 1;
  } while( true );
}



/* ======================================================================
 * 00014c76  vif_cancel_join_timers
 * ====================================================================== */

void vif_cancel_join_timers(int param_1)

{
  int iVar1;
  
  iVar1 = param_1 * 0x3b0 + DAT_00014d34;
  if (*(char *)(iVar1 + 0x18) != '\x02') {
    *(uint *)(iVar1 + 0x1c) = *(uint *)(iVar1 + 0x1c) & 0xfffffeff;
  }
  *(uint *)(iVar1 + 0x1c) = *(uint *)(iVar1 + 0x1c) & 0xfffffdff;
  timer_cancel(iVar1 + 0xc4);
  timer_cancel(DAT_00014d3c + -0x20);
  return;
}



/* ======================================================================
 * 00014cae  vif_cancel_join_timer
 * ====================================================================== */

void vif_cancel_join_timer(int param_1)

{
  timer_cancel(param_1 * 0x3b0 + DAT_00014d34 + 0xc4);
  return;
}



/* ======================================================================
 * 00014cc2  vif_teardown
 * ====================================================================== */

void vif_teardown(uint param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  iVar4 = param_1 * 0x3b0 + DAT_00014d34;
  if (*(char *)(iVar4 + 0x19) != '\0') {
    *(undefined1 *)(iVar4 + 0x32) = 0;
    *(undefined2 *)(iVar4 + 0x2c) = 0;
    *(undefined2 *)(iVar4 + 0x2e) = 0;
    if ((param_1 < 2) && (*(char *)(iVar4 + 0x3c6) != '\0')) {
      *(undefined1 *)(iVar4 + 0x3c6) = 0;
      *(byte *)(iVar4 + 0x38) = *(byte *)(iVar4 + 0x38) ^ 0x80;
    }
    if ((*(uint *)(iVar4 + 0x1c) & 9) != 0) {
      vif_cancel_join_timer(param_1);
    }
    if (*(int *)(iVar4 + 0x1c) << 0xe < 0) {
      vif_stop_beaconing(param_1);
      evt_flags_clear(0x40);
    }
    if (param_1 < 2) {
      link_slot_free(*(undefined1 *)(iVar4 + 0x12a));
      ap_unmap_link(param_1,0);
      vif_reset_all_state(param_1);
    }
    link_deactivate(param_1);
    if ((*(uint *)(iVar4 + 0x1c) & 9) != 0) {
      vif_cancel_join_timers(param_1);
    }
    lmc_msg_complete_dispatch(iVar4 + 0x44);
    if (param_1 == 2) {
      timer_cancel(DAT_0001516c);
      *DAT_00015170 = 0;
    }
    syn_start_release_channel_use(iVar4 + 0x18);
    *(undefined1 *)(iVar4 + 0x19) = 0;
    *(undefined4 *)(iVar4 + 0x1c) = 0;
    lmc_recompute_vif_roles();
    iVar1 = DAT_00015174;
    iVar4 = DAT_0001516c;
    uVar2 = 0;
    *(undefined1 *)(DAT_0001516c + 0x24) = 0;
    do {
      iVar3 = uVar2 * 0x3b0 + iVar1;
      if (*(char *)(iVar3 + 0x19) != '\0') {
        *(byte *)(iVar4 + 0x24) = (byte)(1 << *(sbyte *)(iVar3 + 0x22)) | *(byte *)(iVar4 + 0x24);
      }
      iVar3 = DAT_0001516c;
      uVar2 = uVar2 + 1;
    } while (uVar2 < 3);
    if (*(char *)(iVar4 + 0x24) == '\0') {
      *(undefined2 *)(iVar1 + 2) = 0;
      *(undefined1 *)(iVar3 + -0x62) = 0;
      *(undefined1 *)(iVar3 + -100) = 0;
    }
  }
  return;
}



/* ======================================================================
 * 00014dee  syn_start_wsm_start
 * ====================================================================== */

/* syn_start_wsm_start(struct wsm_start *arg) -- WSM 0x0017 START backend,
   syn_start.c.  Returns 0 ok, 8 rejected, 0x16 busy.
   
   Field mapping is byte-for-byte cw1200's struct wsm_start:
     +0x00 mode           -> low 2 bits select AP(0) / P2P GO(1) / P2P dev(2) / 3
                             bit 4 selects one of two 6-byte parameter sets
                             copied from MIB 0x1046 storage into vif+0x3C..0x40
     +0x01 band           -> vif[0x22]
     +0x02 channel_number -> vif[0x42]
     +0x04 ct_window      -> ONLY read when (mode & 0xF) == 1 (P2P GO)
     +0x08 beacon_interval-> vif[0x118] = value << 10   (TU -> us)
     +0x0C dtim_period    -> vif[0x110], and vif[0x164] = dtim - 1
     +0x0D preamble       -> vif[0x23]
     +0x0E probe_delay    -> NEVER READ
     +0x0F ssid_len       -> vif[0xEC]
     +0x10 ssid[32]       -> vif[0xF0]
     +0x30 basic_rate_set -> vif[0x28]   (stored raw, NOT validated -- an empty
                             basic rate set is accepted silently)
   The beacon-interval / DTIM / SSID block is only applied when if_id < 2.
   
     if (if_id > 1 && (mode & 3) != 2) return 8;
         -> interface 2 is reserved for P2P device mode.
   
     if (vif[0x30] != 0) return 0x16;   /* still busy */
         -> wsm_h_17_start SUPPRESSES the confirm for status 0x16, so the host
            sees a COMMAND TIMEOUT rather than an error status.  Worth knowing
            when debugging a hung WSM_START.
   
   ORDERING REQUIREMENT: the beacon template (MIB 0x1002) must be written
   BEFORE this call.  FUN_00014924, reached from here, asserts
   syn_start.c:1147 code 0x2F when the template length at +10 is zero -- the
   recorded AP-mode field crash. */

undefined4 syn_start_wsm_start(byte *param_1)

{
  byte bVar1;
  undefined4 uVar2;
  int iVar3;
  int iVar4;
  undefined1 *puVar5;
  byte bVar6;
  uint uVar7;
  bool bVar8;
  uint local_18;
  
  uVar7 = (uint)*(byte *)(DAT_00015178 + 0x16);
  bVar1 = *param_1;
  bVar6 = bVar1 & 3;
  if ((1 < uVar7) && (bVar6 != 2)) {
    return 8;
  }
  iVar4 = uVar7 * 0x3b0 + DAT_00015174;
  puVar5 = (undefined1 *)(iVar4 + 0x18);
  if (*(short *)(iVar4 + 0x30) != 0) {
    evt_flags_set(DAT_0001517c,0x200000);
    evt_flags_set(DAT_0001517c,0x100000);
    *(undefined1 *)(DAT_0001516c + 0x27) = 1;
    timer_start(DAT_0001516c + -0x14,0x100);
    return 0x16;
  }
  vif_teardown(uVar7);
  *(byte *)(iVar4 + 0x22) = param_1[1];
  *(undefined2 *)(iVar4 + 0x42) = *(undefined2 *)(param_1 + 2);
  *(byte *)(iVar4 + 0x23) = param_1[0xd];
  *(undefined4 *)(iVar4 + 0x28) = *(undefined4 *)(param_1 + 0x30);
  if (uVar7 < 2) {
    *(byte *)(iVar4 + 0x110) = param_1[0xc];
    *(int *)(iVar4 + 0x118) = *(int *)(param_1 + 8) << 10;
    *(char *)(iVar4 + 0x164) = *(char *)(iVar4 + 0x110) + -1;
    *(uint *)(iVar4 + 0xec) = (uint)param_1[0xf];
    fw_memcpy((void *)(iVar4 + 0xf0),param_1 + 0x10,(uint)param_1[0xf]);
  }
  *(undefined1 *)(iVar4 + 0x1b) = 0;
  bVar8 = (int)((uint)*param_1 << 0x1b) < 0;
  if (bVar8) {
    *(undefined1 *)(iVar4 + 0x1b) = 1;
  }
  local_18 = (uint)bVar8;
  if ((*param_1 & 0xf) == 1) {
    p2p_set_alt_mac_addr(*(int *)(param_1 + 4) == -1,uVar7);
  }
  iVar3 = local_18 * 6 + DAT_00015180;
  *(undefined2 *)(iVar4 + 0x3c) = *(undefined2 *)(iVar3 + 0x454);
  *(undefined2 *)(iVar4 + 0x3e) = *(undefined2 *)(iVar3 + 0x456);
  *(undefined2 *)(iVar4 + 0x40) = *(undefined2 *)(iVar3 + 0x458);
  if ((bVar1 & 3) == 0) {
    uVar2 = 4;
LAB_00014ede:
    *(undefined4 *)(iVar4 + 0x1c) = uVar2;
  }
  else {
    if (bVar6 == 1) {
      uVar2 = 6;
      goto LAB_00014ede;
    }
    if (bVar6 != 3) {
      *(undefined4 *)(iVar4 + 0x1c) = 0x40;
      *puVar5 = 0;
      if ((*param_1 & 0x42) != 0) {
        *(undefined4 *)(iVar4 + 0x1c) = 0xc0;
      }
      goto LAB_00014f08;
    }
    *(undefined4 *)(iVar4 + 0x1c) = 0x8000;
    uVar2 = 7;
  }
  *puVar5 = (char)uVar2;
LAB_00014f08:
  iVar4 = vif_apply_join_config(uVar7);
  if (iVar4 == 0) {
    return 8;
  }
  if (((*param_1 & 0xf) == 0) || ((*param_1 & 0xf) == 1)) {
    timer_entry_init(DAT_00015188,DAT_00015184,puVar5);
  }
  syn_start_finish_vif();
  return 0;
}



/* ======================================================================
 * 00014f36  join_send_confirm_with_tsf
 * ====================================================================== */

void join_send_confirm_with_tsf(undefined4 param_1)

{
  short local_20 [2];
  short asStack_1c [2];
  short asStack_18 [2];
  short asStack_14 [2];
  undefined4 local_10;
  int local_c;
  int local_8;
  
  local_10 = param_1;
  phy_get_tx_power_range(asStack_14,asStack_18,asStack_1c,local_20);
  if (*(char *)((uint)*(byte *)(DAT_00015178 + 0x17) * 0x3b0 + DAT_00015174 + 0x22) == '\0') {
    local_20[0] = asStack_18[0];
    asStack_1c[0] = asStack_14[0];
  }
  local_c = (int)asStack_1c[0];
  local_8 = (int)local_20[0];
  hif_send_confirm_status16(&local_10);
  return;
}



/* ======================================================================
 * 00014f84  join_complete_sta
 * ====================================================================== */

void join_complete_sta(void)

{
  uint *puVar1;
  int iVar2;
  undefined4 in_r3;
  uint uVar3;
  
  puVar1 = DAT_00015190;
  uVar3 = (uint)*(byte *)(DAT_00015178 + 0x17);
  iVar2 = uVar3 * 0x3b0 + DAT_00015174;
  *DAT_0001518c = 0;
  *(undefined1 *)(DAT_00015178 + 0xd8) = 0;
  *puVar1 = *puVar1 & 0xfffffeff | 0x100;
  dup_cache_invalidate_by_mac(uVar3,iVar2 + 0x3c,0,in_r3,in_r3);
  vif_enter_operating_state(uVar3);
  mac_set_state3();
  if (*(int *)(iVar2 + 0x1c) << 0x1c < 0) {
    timer_start(DAT_0001516c + -0x28,*(int *)(DAT_00015178 + 0x10) << 10);
  }
  else {
    syn_scan_rearm_probe_timer();
  }
  *DAT_00015194 = 0;
  *(undefined1 *)(iVar2 + 0x110) = 1;
  uVar3 = *(uint *)(iVar2 + 0x1c);
  if ((int)(uVar3 << 0x14) < 0) {
    if (-1 < (int)(uVar3 << 0x13)) {
      *puVar1 = *puVar1 & 0xfffffeff;
    }
    *(uint *)(iVar2 + 0x1c) = uVar3 | 0x100;
    join_send_confirm_with_tsf(0);
  }
  return;
}



/* ======================================================================
 * 0001500a  start_complete_ap
 * ====================================================================== */

void start_complete_ap(int param_1)

{
  ushort uVar1;
  ushort *puVar2;
  
  puVar2 = DAT_00015174;
  if (((*DAT_00015174 & 0x7f) == 0x13) || ((*DAT_00015174 & 0x7f) == 0x17)) {
    measure_state_init();
  }
  *(uint *)(puVar2 + param_1 * 0x1d8 + 0xe) = *(uint *)(puVar2 + param_1 * 0x1d8 + 0xe) | 0x100;
  uVar1 = puVar2[param_1 * 0x1d8 + 0x8b];
  *(ushort *)(DAT_00015198 + 0x12) = uVar1;
  mac_set_ps_bit(param_1,uVar1 != 0);
  syn_start_load_beacon_template(param_1,0);
  vif_enter_operating_state(param_1);
  mac_set_state3();
  if (*(int *)(puVar2 + param_1 * 0x1d8 + 0xe) << 0x15 < 0) {
    syn_scan_maybe_send_probe(param_1);
  }
  join_send_confirm_with_tsf(0);
  *(uint *)(puVar2 + param_1 * 0x1d8 + 0xe) =
       *(uint *)(puVar2 + param_1 * 0x1d8 + 0xe) & 0xfffbffff | 0x20000;
  beacon_schedule_next(param_1);
  return;
}



/* ======================================================================
 * 00015088  wsm_h_0b_join_impl
 * ====================================================================== */

undefined4
wsm_h_0b_join_impl(char *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  undefined4 uVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  undefined1 *puVar6;
  uint uVar7;
  
  uVar7 = (uint)*(byte *)(DAT_00015178 + 0x17);
  if (1 < uVar7) {
    return 8;
  }
  iVar5 = uVar7 * 0x3b0 + DAT_00015174;
  puVar6 = (undefined1 *)(iVar5 + 0x18);
  if (*(short *)(iVar5 + 0x30) != 0) {
    evt_flags_set(DAT_0001517c,0x200000);
    evt_flags_set(DAT_0001517c,0x100000);
    iVar5 = DAT_0001516c;
    *(undefined1 *)(DAT_0001516c + 0x26) = 1;
    timer_start(iVar5 + -0x14,0x100);
    return 0;
  }
  vif_teardown(uVar7);
  *(char *)(iVar5 + 0x22) = param_1[1];
  *(int *)(iVar5 + 0x118) = *(int *)(param_1 + 0x34) << 10;
  *(undefined2 *)(iVar5 + 0x42) = *(undefined2 *)(param_1 + 2);
  *(undefined2 *)(iVar5 + 0x116) = *(undefined2 *)(param_1 + 10);
  *(char *)(iVar5 + 0x23) = param_1[0xc];
  *(char *)(iVar5 + 0x110) = param_1[0xe];
  *(undefined4 *)(iVar5 + 0x28) = *(undefined4 *)(param_1 + 0x38);
  *(undefined4 *)(iVar5 + 0xec) = *(undefined4 *)(param_1 + 0x10);
  fw_memcpy((void *)(iVar5 + 0xf0),param_1 + 0x14,*(int *)(param_1 + 0x10));
  *(undefined2 *)(iVar5 + 0x3c) = *(undefined2 *)(param_1 + 4);
  *(undefined2 *)(iVar5 + 0x3e) = *(undefined2 *)(param_1 + 6);
  *(undefined2 *)(iVar5 + 0x40) = *(undefined2 *)(param_1 + 8);
  *(undefined1 *)(iVar5 + 0x1b) = 0;
  if (*param_1 == '\0') {
    *puVar6 = 2;
    *(undefined4 *)(iVar5 + 0x1c) = 8;
  }
  else {
    if ((param_1[0xf] & 1U) == 0) {
      if ((int)((uint)(byte)param_1[0xf] << 0x1e) < 0) {
        *puVar6 = 5;
        *(undefined4 *)(iVar5 + 0x1c) = 3;
        *(undefined1 *)(iVar5 + 0x1b) = 4;
        p2p_set_alt_mac_addr(((byte)param_1[0xf] & 0x7f) >> 6,uVar7,0,3,param_4);
        goto LAB_0001515c;
      }
      *puVar6 = 1;
      *(undefined4 *)(iVar5 + 0x1c) = 1;
    }
    else {
      *puVar6 = 3;
      *(undefined4 *)(iVar5 + 0x1c) = 0x10;
    }
    *(undefined1 *)(iVar5 + 0x1b) = 4;
  }
LAB_0001515c:
  if (param_1[0xd] == '\0') {
    if (-1 < (int)((uint)(byte)param_1[0xf] << 0x1d)) goto LAB_000151ac;
    uVar1 = *(uint *)(iVar5 + 0x1c);
    uVar3 = 0x800;
  }
  else {
    uVar1 = *(uint *)(iVar5 + 0x1c);
    uVar3 = 0x400;
  }
  *(uint *)(iVar5 + 0x1c) = uVar1 | uVar3;
LAB_000151ac:
  if ((int)((uint)(byte)param_1[0xf] << 0x1a) < 0) {
    *(uint *)(iVar5 + 0x1c) = *(uint *)(iVar5 + 0x1c) | 0x1000;
  }
  if ((int)((uint)(byte)param_1[0xf] << 0x1b) < 0) {
    *(byte *)(iVar5 + 0x1b) = *(byte *)(iVar5 + 0x1b) | 1;
  }
  if ((*(uint *)(iVar5 + 0x1c) & 1) != 0) {
    uVar1 = 0;
    do {
      iVar4 = uVar1 * 0x3b0 + DAT_0001559c;
      if (((*(char *)(iVar4 + 0x19) != '\0') &&
          (*(short *)(iVar4 + 0x42) != *(short *)(iVar5 + 0x42))) &&
         (-1 < (int)(*(uint *)(iVar5 + 0x1c) << 0x15))) {
        return 2;
      }
      uVar1 = uVar1 + 1;
    } while (uVar1 < 3);
  }
  iVar4 = vif_apply_join_config(uVar7);
  if (iVar4 == 0) {
    return 8;
  }
  if ((param_1[0xf] & 1U) == 0) {
    hif_dbg_ctx_set(iVar5 + 0x3c);
  }
  if (*(int *)(iVar5 + 0x1c) << 0x15 < 0) {
    uVar2 = 100;
  }
  else {
    uVar2 = 2000;
  }
  *(undefined4 *)(DAT_000155a0 + 0x30) = uVar2;
  vif_reset_all_state(*(undefined1 *)(iVar5 + 0x1a));
  if (*(int *)(iVar5 + 0x1c) << 0x1c < 0) {
    start_complete_ap(uVar7);
  }
  else {
    join_complete_sta();
  }
  if (*(int *)(iVar5 + 0x1c) << 0x1e < 0) {
    ps_resync_beacon_state(uVar7);
  }
  return 0;
}



/* ======================================================================
 * 0001524e  phy_recompute_rate_cfg
 * ====================================================================== */

void phy_recompute_rate_cfg(byte *param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined2 *puVar1;
  undefined2 uVar2;
  uint uVar3;
  byte bVar4;
  int iVar5;
  undefined2 *puVar6;
  uint uVar7;
  int iVar8;
  
  iVar8 = DAT_000155a4;
  puVar1 = DAT_0001559c;
  uVar7 = (uint)*(byte *)(DAT_0001559c + 5);
  puVar6 = DAT_0001559c + uVar7 * 0x1d8 + 0xc;
  if ((*param_1 & 1) != 0) {
    bVar4 = param_1[1];
    *(byte *)((int)DAT_0001559c + uVar7 * 0x3b0 + 0x23) = bVar4;
    *(byte *)(puVar1 + 2) = bVar4;
    *(undefined1 *)(iVar8 + 5) = *(undefined1 *)((int)puVar1 + uVar7 * 0x3b0 + 0x23);
  }
  if ((int)((uint)*param_1 << 0x1e) < 0) {
    uVar3 = 0;
    iVar8 = uVar7 * 0x40 + DAT_000155a8;
    do {
      iVar5 = iVar8 + uVar3 * 8;
      bVar4 = *(byte *)(iVar5 + 1);
      if (bVar4 != 0xff) {
        if (param_1[2] == 1) {
          bVar4 = bVar4 | 0x80;
        }
        else {
          bVar4 = bVar4 & 0x7f;
        }
        *(byte *)(iVar5 + 1) = bVar4;
      }
      uVar3 = uVar3 + 1 & 0xff;
    } while (uVar3 < 8);
  }
  if ((int)((uint)*param_1 << 0x1d) < 0) {
    *(undefined4 *)(puVar1 + uVar7 * 0x1d8 + 0x14) = *(undefined4 *)(param_1 + 4);
  }
  if ((int)((uint)*param_1 << 0x1c) < 0) {
    *(byte *)(uVar7 * 0x98 + DAT_000155ac + 0x490) = param_1[3];
  }
  if ((*param_1 & 5) != 0) {
    uVar2 = phy_build_rate_cfg(*(undefined1 *)(puVar1 + uVar7 * 0x1d8 + 0x11),
                               *(uint *)(DAT_000155b0 + 0x30) |
                               *(uint *)(puVar1 + uVar7 * 0x1d8 + 0x14),
                               *(undefined1 *)((int)puVar1 + uVar7 * 0x3b0 + 0x23),
                               puVar1[uVar7 * 0x1d8 + 0x21],param_4);
    iVar8 = DAT_000155a4;
    *puVar1 = uVar2;
    *(undefined2 *)(iVar8 + 2) = uVar2;
    puVar1[uVar7 * 0x1d8 + 0x10] = uVar2;
    iVar5 = DAT_000155b4;
    *(undefined2 *)(DAT_000155b4 + 0x1a) = *(undefined2 *)(iVar8 + 2);
    *(undefined1 *)(iVar5 + 0x19) = *(undefined1 *)((int)puVar1 + uVar7 * 0x3b0 + 0x23);
    vif_set_basic_rates(uVar7,*(undefined4 *)(puVar1 + uVar7 * 0x1d8 + 0x14));
    pas_pick_lowest_rate_from_mask(puVar6);
  }
  return;
}



/* ======================================================================
 * 00015306  bss_loss_keepalive_retry
 * ====================================================================== */

void bss_loss_keepalive_retry(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined1 uVar1;
  byte bVar2;
  undefined4 local_1c;
  uint local_18;
  
  uVar1 = *(undefined1 *)(param_1 + 2);
  if (((*(uint *)(param_1 + 4) & 0x108) == 0x100) && (-1 < (int)(*(uint *)(param_1 + 4) << 3))) {
    bVar2 = *(byte *)(param_1 + 0x3ac);
    local_1c = param_3;
    local_18 = param_4;
    if ((((bVar2 & 0xf) != 4) || (bVar2 >> 4 < 2)) && (bVar2 >> 4 < 10)) {
      *(byte *)(param_1 + 0x3ac) = ((bVar2 >> 4) + 1) * '\x10' + '\x04';
      tx_send_null_data(uVar1,2,bVar2 & 0xf,param_4,param_2);
      timer_start(param_1 + 0xac,DAT_000155b8);
      return;
    }
    timer_cancel(param_1 + 0xac);
    *(uint *)(param_1 + 4) = *(uint *)(param_1 + 4) | 0x200;
    local_1c = 1;
    local_18 = (uint)*(byte *)(param_1 + 0x3ac) | (*(uint *)(param_1 + 0x3a8) & 0xff) << 8;
    ind_0805_event_a(uVar1,&local_1c);
    *(undefined1 *)(param_1 + 0x3ac) = 0;
  }
  return;
}



/* ======================================================================
 * 00015394  join_fail_teardown
 * ====================================================================== */

void join_fail_teardown(undefined4 param_1)

{
  uint uVar1;
  int iVar2;
  
  *DAT_000155bc = *DAT_000155bc & 0xfffffeff;
  uVar1 = (uint)*(byte *)(DAT_000155a0 + 0x37);
  iVar2 = *(int *)(uVar1 * 0x3b0 + DAT_0001559c + 0x1c);
  vif_teardown(uVar1);
  if (iVar2 << 0x13 < 0) {
    ind_080F_join_complete_a(uVar1,param_1);
    return;
  }
  join_send_confirm_with_tsf(param_1);
  return;
}



/* ======================================================================
 * 000153d2  join_retry_or_fail
 * ====================================================================== */

void join_retry_or_fail(void)

{
  byte bVar1;
  uint uVar2;
  
  uVar2 = *(uint *)(DAT_000155b0 + 0x40);
  if ((int)(uVar2 << 0x1d) < 0) {
    *(uint *)(DAT_000155b0 + 0x40) = uVar2 & 0xfffffffb;
  }
  else {
    if (-1 < *DAT_000155bc << 0x17) {
      return;
    }
    bVar1 = *(char *)(DAT_000155a0 + 0xf8) + 1;
    *(byte *)(DAT_000155a0 + 0xf8) = bVar1;
    if (2 < bVar1) {
      join_fail_teardown(7);
      return;
    }
  }
  syn_scan_rearm_probe_timer();
  return;
}



/* ======================================================================
 * 0001540e  join_timeout
 * ====================================================================== */

void join_timeout(void)

{
  if (*DAT_000155bc << 0x17 < 0) {
    timer_cancel(DAT_000155c0);
    join_fail_teardown(7);
  }
  return;
}



/* ======================================================================
 * 00015426  scan_kick_on_rx
 * ====================================================================== */

void scan_kick_on_rx(int param_1)

{
  if (((**(ushort **)(param_1 + 0x1c) & 0xff) != 0x50) && (*(char *)(DAT_000155b0 + 0x34) != '\0'))
  {
    evt_flags_set(DAT_000155c4,0x400);
  }
  return;
}



/* ======================================================================
 * 00015458  wsm_h_join_apply
 * ====================================================================== */

void wsm_h_join_apply(byte *param_1)

{
  byte bVar1;
  undefined2 *puVar2;
  int iVar3;
  undefined2 uVar4;
  
  iVar3 = DAT_000155c8;
  puVar2 = DAT_0001559c;
  bVar1 = *(byte *)(DAT_0001559c + 5);
  if ((*param_1 & 1) == 0) {
    fw_memcpy((void *)(DAT_000155c8 + 0xc),param_1,8);
    *(uint *)(puVar2 + (uint)bVar1 * 0x1d8 + 0x1de) = (uint)*(ushort *)(param_1 + 2);
    *(uint *)(puVar2 + (uint)bVar1 * 0x1d8 + 0x1e0) = (uint)param_1[1];
    uVar4 = phy_build_rate_cfg(*(undefined1 *)(puVar2 + (uint)bVar1 * 0x1d8 + 0x11),
                               *(uint *)(DAT_000155b0 + 0x30) |
                               *(uint *)(puVar2 + (uint)bVar1 * 0x1d8 + 0x14),
                               *(undefined1 *)((int)puVar2 + (uint)bVar1 * 0x3b0 + 0x23),
                               puVar2[(uint)bVar1 * 0x1d8 + 0x21]);
    *puVar2 = uVar4;
    if (-1 < *(int *)(puVar2 + (uint)bVar1 * 0x1d8 + 0xe) << 0x1b) {
      phy_set_cfg_word(*(undefined2 *)(iVar3 + 0xe));
    }
    *DAT_000155bc = *DAT_000155bc | 0x20000;
    ps_mark_awake(*(undefined1 *)(puVar2 + 5));
  }
  else {
    *(byte *)(DAT_000155c8 + 0xd) = param_1[1];
    *(uint *)(puVar2 + (uint)bVar1 * 0x1d8 + 0x1e0) = (uint)param_1[1];
  }
  *(uint *)(puVar2 + (uint)bVar1 * 0x1d8 + 0xe) =
       *(uint *)(puVar2 + (uint)bVar1 * 0x1d8 + 0xe) & 0xfffffdff;
  vif_bss_event_timer_update(*(undefined1 *)(puVar2 + 5));
  return;
}



/* ======================================================================
 * 000154e0  mac_program_channel_for_vifs
 * ====================================================================== */

void mac_program_channel_for_vifs
               (uint param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  ushort uVar4;
  undefined1 local_20;
  undefined1 local_1f [3];
  undefined4 local_1c;
  undefined4 uStack_18;
  
  *(short *)(DAT_0001559c + 2) = (short)param_1;
  iVar3 = 0;
  local_1f = SUB43((uint)param_2 >> 8,0);
  _local_20 = CONCAT31(local_1f,1);
  uVar2 = 0;
  local_1c = param_3;
  uStack_18 = param_4;
  do {
    iVar1 = uVar2 * 0x3b0 + DAT_0001559c;
    if ((*(char *)(iVar1 + 0x19) != '\0') && (*(ushort *)(iVar1 + 0x42) == param_1)) {
      *(undefined1 *)(iVar1 + 0x19) = 2;
      _local_20 = CONCAT22(*(undefined2 *)(iVar1 + 0x20),
                           CONCAT11(*(undefined1 *)(iVar1 + 0x23),local_20));
      *(ushort *)(iVar1 + 0x2e) = *(ushort *)(iVar1 + 0x2c);
      uVar4 = *(ushort *)(iVar1 + 0x2c) & (~*(ushort *)(iVar1 + 0x15c) | *(ushort *)(iVar1 + 0x160))
      ;
      *(ushort *)(iVar1 + 0x2e) = uVar4;
      *(ushort *)(iVar1 + 0x2e) = *(ushort *)(iVar1 + 0x15e) | uVar4;
      *(char *)((int)&local_1c + iVar3 + 3) = (char)uVar2;
      iVar3 = iVar3 + 1;
      local_1c._0_3_ = CONCAT12((char)iVar3,(short)param_1);
    }
    if (*(char *)(iVar1 + 0x18) == '\a') {
      _local_20 = CONCAT31(local_1f,5);
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 3);
  mac_apply_channel_and_vif_config(&local_20);
  return;
}



/* ======================================================================
 * 0001555a  mac_program_channel_for_vifs_ex
 * ====================================================================== */

void mac_program_channel_for_vifs_ex
               (uint param_1,int param_2,undefined1 *param_3,undefined4 param_4)

{
  undefined1 *puVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  ushort uVar5;
  undefined1 uStack_20;
  undefined1 auStack_1f [3];
  undefined4 uStack_1c;
  undefined4 uStack_18;
  
  puVar1 = DAT_000155cc;
  uStack_1c = param_3;
  if (param_2 == 2) {
    param_2 = 0;
    *DAT_000155cc = 1;
    uStack_1c = puVar1;
  }
  *(short *)(DAT_0001559c + 2) = (short)param_1;
  iVar4 = 0;
  auStack_1f = SUB43((uint)param_2 >> 8,0);
  _uStack_20 = CONCAT31(auStack_1f,1);
  uVar3 = 0;
  uStack_18 = param_4;
  do {
    iVar2 = uVar3 * 0x3b0 + DAT_0001559c;
    if ((*(char *)(iVar2 + 0x19) != '\0') && (*(ushort *)(iVar2 + 0x42) == param_1)) {
      *(undefined1 *)(iVar2 + 0x19) = 2;
      _uStack_20 = CONCAT22(*(undefined2 *)(iVar2 + 0x20),
                            CONCAT11(*(undefined1 *)(iVar2 + 0x23),uStack_20));
      *(ushort *)(iVar2 + 0x2e) = *(ushort *)(iVar2 + 0x2c);
      uVar5 = *(ushort *)(iVar2 + 0x2c) & (~*(ushort *)(iVar2 + 0x15c) | *(ushort *)(iVar2 + 0x160))
      ;
      *(ushort *)(iVar2 + 0x2e) = uVar5;
      *(ushort *)(iVar2 + 0x2e) = *(ushort *)(iVar2 + 0x15e) | uVar5;
      *(char *)((int)&uStack_1c + iVar4 + 3) = (char)uVar3;
      iVar4 = iVar4 + 1;
      uStack_1c._0_3_ = CONCAT12((char)iVar4,(short)param_1);
    }
    if (*(char *)(iVar2 + 0x18) == '\a') {
      _uStack_20 = CONCAT31(auStack_1f,5);
    }
    uVar3 = uVar3 + 1;
  } while (uVar3 < 3);
  mac_apply_channel_and_vif_config(&uStack_20);
  return;
}



/* ======================================================================
 * 000155d0  syn_scan_rearm_probe_timer
 * ====================================================================== */

void syn_scan_rearm_probe_timer(void)

{
  timer_start(DAT_000155f0,*(int *)(DAT_000155ec + 0x30) << 10);
  syn_scan_maybe_send_probe(*(undefined1 *)(DAT_000155ec + 0x37));
  return;
}



/* ======================================================================
 * 00015650  template_replace_ie
 * ====================================================================== */

/* template_replace_ie(if_id, ie, template_type) -- insert, replace or delete a
   single IE inside a live frame template, shifting the remainder to fit.
   Returns 0 ok, 1 template empty, 2 bad type, 3 IE id mismatch, 4 would overflow,
   5 if_id > 1.
   
   Body start and size cap depend on the template type:
   
     type 0 (PROBE_REQUEST)  body = data+0x18   cap = 0x100 = 256
     type 1 (BEACON)         body = data+0x24   cap = 700
     type 5 (PROBE_RESPONSE) body = data+0x24   cap = 700
     anything else           -> return 2
   
   *** NOTE THE SECOND, DIFFERENT SIZE LIMIT. ***
   The MIB 0x1002 TEMPLATE_FRAME write path validates against g_template_max_len
   (0x0001B380), which gives **694** for types 1 and 5.  This function instead
   checks against **700**.  Both are real and they are checked on different paths:
   
     * host writes a whole template via MIB 0x1002   -> limit 694
     * firmware grows a template by inserting an IE  -> limit 700
   
   So a beacon can legitimately reach 700 bytes after the firmware inserts its own
   IEs (e.g. the P2P NoA attribute, see beacon_insert_p2p_noa_ie), even though the
   host could not have written 700 directly.  The 6-byte gap is presumably deliberate
   headroom.  Neither number contradicts the other; earlier notes only recorded 694.
   
   Type 1 also latches two flags while it is here:
     * probe-response/beacon capability bit 0 of ie[4] -> g_ctx+0x15
     * for IE id 5 (TIM) with a zero count, vif+0x3C5 is cleared, else set.
   
   After any successful edit to type 1 it calls syn_start_load_beacon_template to
   push the modified template into the live beacon buffer. */

undefined4 template_replace_ie(uint param_1,byte *param_2,int param_3)

{
  undefined4 uVar1;
  int iVar2;
  byte *pbVar3;
  int iVar4;
  byte *pbVar5;
  int len;
  byte *src;
  int iVar6;
  uint uVar7;
  uint local_2c;
  
  iVar6 = param_1 * 0x3b0 + DAT_000158b8;
  if (param_1 < 2) {
    iVar4 = param_1 * 0x40 + DAT_000158bc + param_3 * 8;
    uVar7 = (uint)*(ushort *)(iVar4 + 2);
    iVar2 = *(int *)(iVar4 + 4);
    if (uVar7 == 0) {
      uVar1 = 1;
    }
    else {
      pbVar5 = (byte *)(iVar2 + uVar7);
      if (param_3 == 0) {
        local_2c = 0x100;
        pbVar3 = (byte *)(iVar2 + 0x18);
        len = uVar7 - 0x18;
      }
      else if (param_3 == 1) {
        local_2c = 700;
        len = uVar7 - 0x24;
        pbVar3 = (byte *)(iVar2 + 0x24);
        if ((param_2[4] & 1) == 0) {
          *(undefined1 *)(DAT_000158c0 + 0x15) = 0;
        }
        else {
          *(undefined1 *)(DAT_000158c0 + 0x15) = 1;
        }
        if ((*(int *)(iVar6 + 0x3a4) << 0x1b < 0) && (*param_2 == 5)) {
          iVar6 = param_1 * 0x3b0 + DAT_000158b8;
          if (*(short *)(param_2 + 5) == 0) {
            *(undefined1 *)(iVar6 + 0x3c5) = 0;
          }
          else {
            *(undefined1 *)(iVar6 + 0x3c5) = 1;
          }
        }
      }
      else {
        if (param_3 != 5) {
          return 2;
        }
        local_2c = 700;
        pbVar3 = (byte *)(iVar2 + 0x24);
        len = uVar7 - 0x24;
      }
      pbVar3 = ie_find(pbVar3,len,(uint)*param_2,0);
      if (pbVar3 == (byte *)0x0) {
        iVar6 = param_2[1] + 2;
        src = pbVar5;
        pbVar3 = pbVar5;
      }
      else {
        if (*pbVar3 != *param_2) {
          return 3;
        }
        src = pbVar3 + pbVar3[1] + 2;
        iVar6 = (uint)param_2[1] - (uint)pbVar3[1];
      }
      if (iVar6 < 1) {
        if (iVar6 < 0) {
          fw_memcpy(pbVar3 + param_2[1] + 2,src,(int)pbVar5 - (int)src & 0xffff);
        }
      }
      else {
        if (local_2c < uVar7 + iVar6) {
          return 4;
        }
        fw_memmove_rev(pbVar3 + param_2[1] + 2);
      }
      fw_memcpy(pbVar3,param_2,param_2[1] + 2);
      *(short *)(iVar4 + 2) = *(short *)(iVar4 + 2) + (short)iVar6;
      if (param_3 == 1) {
        syn_start_load_beacon_template(param_1,0);
      }
      uVar1 = 0;
    }
  }
  else {
    uVar1 = 5;
  }
  return uVar1;
}



/* ======================================================================
 * 00015792  beacon_set_tim_bcast_bit
 * ====================================================================== */

void beacon_set_tim_bcast_bit(int param_1)

{
  int iVar1;
  byte bVar2;
  
  iVar1 = param_1 * 0x70 + DAT_000158c4;
  iVar1 = ie_find_in_frame(*(undefined4 *)(iVar1 + 0x10),*(undefined2 *)(iVar1 + 0x18),5,0);
  if (iVar1 != 0) {
    if (((*(char *)(DAT_000158c0 + 0x14) == '\0') && (*(char *)(DAT_000158c0 + 0x15) == '\0')) ||
       (*(char *)(param_1 * 0x3b0 + DAT_000158b8 + 0x164) != '\0')) {
      bVar2 = *(byte *)(iVar1 + 4) & 0xfe;
    }
    else {
      bVar2 = *(byte *)(iVar1 + 4) | 1;
    }
    *(byte *)(iVar1 + 4) = bVar2;
  }
  return;
}



/* ======================================================================
 * 000157e0  p2p_find_noa_attr
 * ====================================================================== */

undefined4 p2p_find_noa_attr(char *param_1)

{
  uint uVar1;
  byte *pbVar2;
  uint uVar3;
  byte *buf;
  undefined4 uVar4;
  
  uVar1 = DAT_000158c8;
  uVar4 = 0;
  if ((*param_1 == '\x01') || (*param_1 == '\x05')) {
    buf = (byte *)(*(int *)(param_1 + 4) + 0x24);
    uVar3 = *(ushort *)(param_1 + 2) - 0x24;
    while( true ) {
      uVar3 = uVar3 & 0xffff;
      if (uVar3 == 0) {
        return 0;
      }
      pbVar2 = ie_find(buf,uVar3,0xdd,0);
      if (pbVar2 == (byte *)0x0) {
        return 0;
      }
      if (((uint)pbVar2[2] == (uVar1 & 0xff)) && (pbVar2[5] == 9)) break;
      if (uVar3 < ((uint)(pbVar2 + ((uint)pbVar2[1] - (int)buf)) & 0xffff)) {
        return 0;
      }
      uVar3 = uVar3 - ((uint)(pbVar2 + ((uint)pbVar2[1] - (int)buf)) & 0xffff);
      buf = pbVar2 + pbVar2[1] + 2;
    }
    uVar4 = tlv_find_u16len(pbVar2 + 6,pbVar2[1] - 4,0xc);
  }
  return uVar4;
}



/* ======================================================================
 * 00015854  p2p_find_vendor_ie_in_template
 * ====================================================================== */

undefined4 p2p_find_vendor_ie_in_template(char *param_1)

{
  undefined4 uVar1;
  
  uVar1 = 0;
  if ((*param_1 == '\x01') || (*param_1 == '\x05')) {
    uVar1 = ie_find_p2p_vendor(*(int *)(param_1 + 4) + 0x24,*(short *)(param_1 + 2) + -0x24);
  }
  return uVar1;
}



/* ======================================================================
 * 0001587a  template_remove_bytes
 * ====================================================================== */

undefined4 template_remove_bytes(int param_1,void *param_2,int param_3)

{
  ushort uVar1;
  uint n;
  
  uVar1 = *(ushort *)(param_1 + 2);
  n = (*(int *)(param_1 + 4) + (uint)uVar1) - ((int)param_2 + param_3) & 0xffff;
  if (n == 0) {
    fw_memzero(param_2,param_3);
  }
  fw_memcpy(param_2,(void *)((int)param_2 + param_3),n);
  *(ushort *)(param_1 + 2) = uVar1 - (short)param_3;
  return 1;
}



/* ======================================================================
 * 000158cc  wsm_h_03_impl
 * ====================================================================== */

void wsm_h_03_impl(undefined2 *param_1)

{
  undefined4 uVar1;
  undefined2 uVar2;
  int iVar3;
  
  iVar3 = *(int *)(param_1 + 2);
  uVar2 = 0x10;
  if (iVar3 == 1) {
    uVar2 = *param_1;
    uVar1 = fw_read_timer();
    *(undefined4 *)(param_1 + 8) = uVar1;
  }
  *(uint *)(param_1 + 4) = (uint)(iVar3 != 1);
  uVar1 = DAT_000158f8;
  *param_1 = uVar2;
  *(int *)(param_1 + 2) = iVar3;
  param_1[1] = (short)uVar1;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 0001596e  mac_phy_stop_for_exception
 * ====================================================================== */

void mac_phy_stop_for_exception(void)

{
  mac_init_partial();
  phy_rx_disable_and_drain();
  return;
}



/* ======================================================================
 * 00015a30  dbg_console_register_cmds
 * ====================================================================== */

void dbg_console_register_cmds(undefined4 param_1,int *param_2,uint param_3)

{
  int iVar1;
  uint uVar2;
  
  iVar1 = DAT_00015d00;
  uVar2 = 0;
  do {
    if (param_3 <= uVar2) {
      return;
    }
    if (*(uint *)(iVar1 + 0x24) < 0x20) {
      if (*param_2 == 0) {
        return;
      }
      if (param_2[2] == 0) {
        return;
      }
      if (param_2[1] == 0) {
        return;
      }
      *(int **)(*(uint *)(iVar1 + 0x24) * 4 + iVar1 + 0x28) = param_2;
      *(int *)(iVar1 + 0x24) = *(int *)(iVar1 + 0x24) + 1;
    }
    param_2 = param_2 + 3;
    uVar2 = uVar2 + 1;
  } while( true );
}



/* ======================================================================
 * 00015a66  dbg_next_token
 * ====================================================================== */

char * dbg_next_token(undefined4 *param_1)

{
  char cVar1;
  char *pcVar2;
  char *pcVar3;
  
  for (pcVar2 = (char *)*param_1;
      (cVar1 = *pcVar2, pcVar3 = pcVar2, cVar1 != '\0' &&
      (((cVar1 == ' ' || (cVar1 == ',')) || (cVar1 == '\t')))); pcVar2 = pcVar2 + 1) {
  }
  do {
    if (cVar1 == '\0') {
LAB_00015a9e:
      *param_1 = pcVar3;
      return pcVar2;
    }
    if (((cVar1 == ' ') || (cVar1 == ',')) || (cVar1 == '\t')) {
      *pcVar3 = '\0';
      pcVar3 = pcVar3 + 1;
      goto LAB_00015a9e;
    }
    cVar1 = pcVar3[1];
    pcVar3 = pcVar3 + 1;
  } while( true );
}



/* ======================================================================
 * 00015aa4  dbg_console_execute
 * ====================================================================== */

void dbg_console_execute(char *param_1)

{
  int iVar1;
  char *pcVar2;
  int iVar3;
  uint uVar4;
  char *local_18;
  
  local_18 = param_1;
  pcVar2 = (char *)dbg_next_token(&local_18);
  iVar1 = DAT_00015d00;
  if (*pcVar2 != '\0') {
    uVar4 = 0;
    while( true ) {
      if (*(uint *)(iVar1 + 0x24) <= uVar4) {
        if (*param_1 != '?') {
          func_0xfff014ea(s_Invalid_command___s_00015d10,param_1);
          return;
        }
        for (uVar4 = 0; uVar4 < *(uint *)(iVar1 + 0x24); uVar4 = uVar4 + 1) {
          func_0xfff014ea(s__s_00015d04,*(undefined4 *)(*(int *)(uVar4 * 4 + iVar1 + 0x28) + 4));
        }
        func_0xfff014ea(DAT_00015d0c);
        return;
      }
      iVar3 = fw_streq(pcVar2,**(undefined4 **)(uVar4 * 4 + iVar1 + 0x28));
      if (iVar3 != 0) break;
      uVar4 = uVar4 + 1;
    }
    (**(code **)(*(int *)(uVar4 * 4 + iVar1 + 0x28) + 8))(local_18);
  }
  return;
}



/* ======================================================================
 * 00015b22  dbg_console_readline
 * ====================================================================== */

uint * dbg_console_readline(void)

{
  uint *puVar1;
  uint *puVar2;
  int iVar3;
  uint in_r3;
  uint uVar4;
  uint local_10;
  
  puVar1 = DAT_00015d00;
  uVar4 = *DAT_00015d00;
  local_10 = in_r3;
  while( true ) {
    if ((0x4f < uVar4) || (iVar3 = func_0xfff014a0(&local_10), iVar3 == 0)) {
      *puVar1 = uVar4;
      return (uint *)0x0;
    }
    if (((local_10 & 0xff) == 0xd) || (((local_10 & 0xff) == 10 || (uVar4 == 0x4f)))) break;
    *(undefined1 *)((int)puVar1 + uVar4 + 0xa8) = (undefined1)local_10;
    uVar4 = uVar4 + 1;
  }
  *(undefined1 *)((int)puVar1 + uVar4 + 0xa8) = 0;
  puVar2 = DAT_00015d00;
  *puVar1 = 0;
  func_0xfff014ea(&DAT_00015d28);
  return puVar2 + 0x2a;
}



/* ======================================================================
 * 00015b70  task_15b70
 * ====================================================================== */

void task_15b70(void)

{
  int iVar1;
  
  iVar1 = DAT_00015d30;
  if (((*(uint *)(DAT_00015d30 + 4) & 1) == 0) && (-1 < *(int *)(DAT_00015d30 + 4) << 0x1a)) {
    func_0xfff014ea(s_Sleep_mode_is_suspended_for_20s_00015d34);
    timer_start(DAT_00015d00 + 8,DAT_00015d58);
    evt_flags_set((uint *)(iVar1 + 4),0x20);
  }
  func_0xfff013e4(1);
  while (iVar1 = dbg_console_readline(), iVar1 != 0) {
    dbg_console_execute();
  }
  func_0xfff013e4(0);
  return;
}



/* ======================================================================
 * 00015bb4  dbg_console_init
 * ====================================================================== */

void dbg_console_init(void)

{
  sched_register_task(8,DAT_00015d5c);
  dbg_console_register_cmds(s_CMD_LINE_00015d64,DAT_00015d60,6);
  func_0xfff014ea(s_The_command_line_is_supported__T_00015d70);
  timer_entry_init(DAT_00015d00 + 8,DAT_00015da4,0);
  return;
}



/* ======================================================================
 * 00015bdc  dbg_console_poll
 * ====================================================================== */

void dbg_console_poll(void)

{
  int iVar1;
  
  func_0xfff0141a();
  iVar1 = dbg_console_readline();
  if (iVar1 != 0) {
    dbg_console_execute();
  }
  return;
}



/* ======================================================================
 * 00015bf0  dbg_parse_hex
 * ====================================================================== */

int dbg_parse_hex(undefined4 param_1,uint *param_2)

{
  byte *pbVar1;
  byte *pbVar2;
  uint uVar3;
  int iVar4;
  uint uVar5;
  
  uVar5 = 0;
  pbVar1 = (byte *)dbg_next_token();
  if (*pbVar1 == 0) {
    return 0;
  }
  iVar4 = 0;
  pbVar2 = pbVar1;
  do {
    uVar3 = (uint)*pbVar2;
    if (uVar3 == 0) {
      if (iVar4 == 0) {
LAB_00015c50:
        func_0xfff014ea(s_Invalid_hexdecimal___s_00015da8,pbVar1);
      }
      else {
        *param_2 = uVar5;
      }
      return iVar4;
    }
    if (uVar3 - 0x30 < 10) {
      uVar5 = uVar5 << 4 | uVar3 - 0x30;
    }
    else {
      if (uVar3 - 0x61 < 6) {
        iVar4 = -0x57;
      }
      else {
        if (5 < uVar3 - 0x41) {
          iVar4 = 0;
          goto LAB_00015c50;
        }
        iVar4 = -0x37;
      }
      uVar5 = uVar5 << 4 | uVar3 + iVar4;
    }
    iVar4 = 1;
    pbVar2 = pbVar2 + 1;
  } while( true );
}



/* ======================================================================
 * 00015c5a  dbg_parse_udec
 * ====================================================================== */

int dbg_parse_udec(undefined4 param_1,int *param_2)

{
  byte *pbVar1;
  byte *pbVar2;
  uint uVar3;
  int iVar4;
  int iVar5;
  
  iVar5 = 0;
  pbVar1 = (byte *)dbg_next_token();
  if (*pbVar1 == 0) {
    return 0;
  }
  iVar4 = 0;
  pbVar2 = pbVar1;
  do {
    uVar3 = (uint)*pbVar2;
    if (uVar3 == 0) {
      if (iVar4 == 0) {
LAB_00015c9a:
        func_0xfff014ea(s_Invalid_unsigned_decimal___s_00015dc0,pbVar1);
      }
      else {
        *param_2 = iVar5;
      }
      return iVar4;
    }
    if (9 < uVar3 - 0x30) {
      iVar4 = 0;
      goto LAB_00015c9a;
    }
    iVar4 = 1;
    iVar5 = iVar5 * 10 + uVar3 + -0x30;
    pbVar2 = pbVar2 + 1;
  } while( true );
}



/* ======================================================================
 * 00015ca4  dbg_cmd_mem
 * ====================================================================== */

void dbg_cmd_mem(undefined4 param_1)

{
  int iVar1;
  undefined4 uStack_10;
  
  uStack_10 = param_1;
  dbg_parse_hex(&uStack_10,DAT_00015d00 + 0x1c);
  dbg_parse_udec(&uStack_10,DAT_00015d00 + 0x20);
  iVar1 = DAT_00015d00;
  if (*(int *)(DAT_00015d00 + 0x20) == 0) {
    *(undefined4 *)(DAT_00015d00 + 0x20) = 0x100;
  }
  *(uint *)(iVar1 + 0x1c) = *(uint *)(iVar1 + 0x1c) & 0xfffffffc;
  func_0xfff014ea(s_Memory_at__08X__00015de0);
  func_0xfff016f8(*(undefined4 *)(iVar1 + 0x1c),*(undefined4 *)(iVar1 + 0x20));
  *(int *)(iVar1 + 0x1c) = *(int *)(iVar1 + 0x1c) + *(int *)(iVar1 + 0x20);
  return;
}



/* ======================================================================
 * 00015e28  dbg_cmd_halt
 * ====================================================================== */

void dbg_cmd_halt(void)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = DAT_00015ee4;
  *(undefined4 *)(DAT_00015ee4 + 4) = 1;
  func_0xfff014ea(s_The_system_is_in_halt__You_can_s_00015ee8);
  uVar2 = irq_fiq_disable_save();
  while (*(int *)(iVar1 + 4) != 0) {
    dbg_console_poll();
  }
  irq_fiq_restore(uVar2);
  func_0xfff014ea(s_The_system_is_resumed__It_may_no_00015f20);
  return;
}



/* ======================================================================
 * 00015e56  FUN_00015e56
 * ====================================================================== */

void FUN_00015e56(void)

{
  timer_cancel(DAT_00015ee4 + 8);
  evt_flags_set(DAT_00015f54,0x20);
  func_0xfff014ea(s_Sleep_mode_is_supended_untill__g_00015f58);
  return;
}



/* ======================================================================
 * 00015e70  FUN_00015e70
 * ====================================================================== */

void FUN_00015e70(void)

{
  if (*DAT_00015f54 << 0x1a < 0) {
    func_0xfff014ea(DAT_00015f88);
    evt_flags_clear(0x20);
  }
  if (*(int *)(DAT_00015ee4 + 4) == 0) {
    func_0xfff014ea(s_The_system_is_already_running_00015f8c);
    return;
  }
  *(undefined4 *)(DAT_00015ee4 + 4) = 0;
  return;
}



/* ======================================================================
 * 00015e9e  FUN_00015e9e
 * ====================================================================== */

void FUN_00015e9e(void)

{
  int iVar1;
  
  func_0xfff014ea(s_sOsGlobal_at__08x__00015fac,DAT_00015f54 + -4);
  func_0xfff016f8(DAT_00015f54 + -4,0x40);
  func_0xfff014ea(s_sPlatformLocal_at__08x__00015fc4,DAT_00015fc0);
  func_0xfff016f8(DAT_00015fc0,0x34);
  iVar1 = DAT_00015fc0;
  *(undefined4 *)(DAT_00015fc0 + 0x2c) = 0;
  *(undefined4 *)(iVar1 + 0x30) = 0;
  return;
}



/* ======================================================================
 * 00015fe0  dbg_print_help_text
 * ====================================================================== */

void dbg_print_help_text(void)

{
  int *piVar1;
  int iVar2;
  uint uVar3;
  
  piVar1 = DAT_00016048;
  *(undefined1 *)(DAT_00016044 + 0x14) = 1;
  *piVar1 = 0;
  do {
  } while (*piVar1 << 0x13 < 0);
  piVar1[2] = DAT_0001604c;
  iVar2 = DAT_00016050;
  uVar3 = 1;
  do {
    piVar1[2] = (uint)*(byte *)(iVar2 + uVar3);
    uVar3 = uVar3 + 1;
  } while (uVar3 < 0x1ae);
  *piVar1 = (int)rx_buf_free;
  return;
}



/* ======================================================================
 * 00016012  dbg_print_banner
 * ====================================================================== */

void dbg_print_banner(void)

{
  int *piVar1;
  int iVar2;
  uint uVar3;
  
  piVar1 = DAT_00016048;
  *(undefined1 *)(DAT_00016044 + 0x14) = 2;
  *piVar1 = 0;
  do {
  } while (*piVar1 << 0x13 < 0);
  piVar1[2] = DAT_0001604c;
  iVar2 = DAT_00016054;
  uVar3 = 1;
  do {
    piVar1[2] = (uint)*(byte *)(iVar2 + uVar3);
    uVar3 = uVar3 + 1;
  } while (uVar3 < 0x14a);
  *piVar1 = (int)rx_buf_free;
  return;
}



/* ======================================================================
 * 00016058  hif_post_message
 * ====================================================================== */

/* hif_post_message(msg) -- hand a built WSM message to the host interface.
   
     *(u32 *)(HIF + 0x2C) = (u32)msg & 0xF6FFFFFF;      /* message address */
     *(u32 *)(HIF + 0x30) = (*(u16 *)msg + 1) & 0x1FFF | 1;  /* length | doorbell */
   
   The address mask clears bits 27 and 24, i.e. it strips the cacheable/TCM alias
   bits so the DMA sees a physical address.  The length field is the u16 MsgLen from
   the message header plus 1, clamped to 13 bits, with bit 0 set as the "valid"
   doorbell.
   
   This is the lowest-level TX-to-host primitive; every indication and confirm ends
   up here, including exc_build_indication_and_spin's crash dump.  A 13-bit length
   field caps any single host-bound message at 8191 bytes. */

void hif_post_message(ushort *param_1)

{
  ushort uVar1;
  int iVar2;
  
  iVar2 = DAT_00016074;
  uVar1 = *param_1;
  *(uint *)(DAT_00016074 + 0x2c) = (uint)param_1 & 0xf6ffffff;
  *(uint *)(iVar2 + 0x30) = uVar1 + 1 & 0x1fff | 1;
  return;
}



/* ======================================================================
 * 00016078  wsm_h_00_impl
 * ====================================================================== */

/* WSM command 0x0000 -- BULK FIRMWARE MEMORY READ.  Confirm id 0x0400.
   
   Request body:
     +0x04  u32 src address
     +0x08  u16 length      (clamped to 0x400 = 1024 bytes)
     +0x0A  u16 flags       bits 6..7 select a lock around the copy:
                              bit 7 set -> FUN_0000F004 / FUN_0000F018
                              else      -> FUN_0000EFDC / FUN_0000EFF0
   Reply: MsgLen = length + 8, the bytes at +0x08, and +0x02 = 0x400.
   
   The copy is a plain word loop from the caller-supplied pointer, so this
   reads ANY firmware address, up to 1 KB per request.  Together with command
   0x0001 (bulk write) this is a far better memory peek/poke channel than MIB
   0x0009 RW_FW_REG, which is limited to 16 words per request.
   
   Neither mainline nor either vendor tree issues this opcode.  Nothing
   validates the address, so a bad pointer faults the firmware. */

void wsm_h_00_impl(short *param_1)

{
  undefined4 uVar1;
  uint uVar2;
  undefined4 uVar3;
  uint uVar4;
  undefined4 *puVar5;
  short *psVar6;
  
  uVar2 = (uint)(ushort)param_1[5];
  psVar6 = param_1 + 4;
  uVar1 = 0;
  puVar5 = *(undefined4 **)(param_1 + 2);
  uVar4 = (uint)(ushort)param_1[4];
  if (0x400 < (ushort)param_1[4]) {
    uVar4 = 0x400;
  }
  *param_1 = (short)uVar4 + 8;
  if ((uVar2 & 0xff) >> 6 != 0) {
    if ((int)(uVar2 << 0x19) < 0) {
      uVar1 = irq_fiq_disable_save();
    }
    else {
      uVar1 = irq_disable_save();
    }
  }
  for (; 0 < (int)uVar4; uVar4 = uVar4 - 4) {
    uVar3 = *puVar5;
    puVar5 = puVar5 + 1;
    *(undefined4 *)psVar6 = uVar3;
    psVar6 = psVar6 + 2;
  }
  if ((uVar2 & 0xff) >> 6 != 0) {
    if ((int)(uVar2 << 0x19) < 0) {
      irq_fiq_restore();
    }
    else {
      irq_restore(uVar1);
    }
  }
  param_1[1] = 0x400;
  param_1[2] = 0;
  param_1[3] = 0;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 000160e8  wsm_h_01_impl
 * ====================================================================== */

/* WSM command 0x0001 -- BULK FIRMWARE MEMORY WRITE.  Confirm id 0x0401.
   
   Request body:
     +0x04  u32 dst address
     +0x08  u16 length
     +0x0A  u16 flags       same lock selection as command 0x0000
     +0x0C  payload         (source data, offset by dst&3 for alignment)
   Reply: MsgLen = 8, status word zeroed, confirm id 0x0401.
   
   Handles an unaligned destination properly: byte prologue if dst&1,
   halfword if dst&2, then a word loop, then halfword/byte epilogue.
   
   Pairs with command 0x0000 (bulk read).  Unvalidated destination, so this
   can overwrite anything in firmware -- including code, since the image is
   RAM-resident.  Neither driver issues it. */

void wsm_h_01_impl(undefined2 *param_1)

{
  undefined1 uVar1;
  undefined4 uVar2;
  uint uVar3;
  undefined4 uVar4;
  uint uVar5;
  undefined4 *puVar6;
  undefined4 *puVar7;
  undefined4 *puVar8;
  uint uVar9;
  
  uVar3 = (uint)(ushort)param_1[5];
  uVar2 = 0;
  puVar6 = *(undefined4 **)(param_1 + 2);
  uVar5 = (uint)puVar6 & 3;
  puVar8 = (undefined4 *)((int)param_1 + uVar5 + 0xc);
  uVar9 = (ushort)param_1[4] - uVar5;
  if ((uVar3 & 0xff) >> 6 != 0) {
    if ((int)(uVar3 << 0x19) < 0) {
      uVar2 = irq_fiq_disable_save();
    }
    else {
      uVar2 = irq_disable_save();
    }
  }
  puVar7 = puVar6;
  if (((uint)puVar6 & 1) != 0) {
    uVar1 = *(undefined1 *)puVar8;
    puVar8 = (undefined4 *)((int)param_1 + uVar5 + 0xd);
    *(undefined1 *)puVar6 = uVar1;
    puVar7 = (undefined4 *)((int)puVar6 + 1);
  }
  if ((int)puVar6 << 0x1e < 0) {
    *(undefined2 *)puVar7 = *(undefined2 *)puVar8;
    puVar7 = (undefined4 *)((int)puVar7 + 2);
    puVar8 = (undefined4 *)((int)puVar8 + 2);
  }
  for (; 3 < (int)uVar9; uVar9 = uVar9 - 4) {
    uVar4 = *puVar8;
    puVar8 = puVar8 + 1;
    *puVar7 = uVar4;
    puVar7 = puVar7 + 1;
  }
  if ((int)(uVar9 << 0x1e) < 0) {
    *(undefined2 *)puVar7 = *(undefined2 *)puVar8;
    puVar7 = (undefined4 *)((int)puVar7 + 2);
    puVar8 = (undefined4 *)((int)puVar8 + 2);
  }
  if ((uVar9 & 1) != 0) {
    *(undefined1 *)puVar7 = *(undefined1 *)puVar8;
  }
  if ((uVar3 & 0xff) >> 6 != 0) {
    if ((int)(uVar3 << 0x19) < 0) {
      irq_fiq_restore();
    }
    else {
      irq_restore(uVar2);
    }
  }
  *(undefined4 *)(param_1 + 2) = 0;
  *param_1 = 8;
  param_1[1] = (short)DAT_00016198;
  hif_send_msg_to_host(param_1);
  return;
}



/* ======================================================================
 * 00016182  wsm_cmd_unsupported
 * ====================================================================== */

void wsm_cmd_unsupported(undefined2 *param_1)

{
  param_1[1] = param_1[1] | 0x400;
  *param_1 = 4;
  hif_send_msg_to_host();
  return;
}



/* ======================================================================
 * 0001619c  irq_register_handler
 * ====================================================================== */

void irq_register_handler(uint param_1,undefined4 param_2)

{
  if (0x1f < param_1) {
    fw_assert(s_irq_c_000161d8,0x9c,0x16);
  }
  *(undefined4 *)(DAT_000161e0 + (0x1f - param_1) * 4) = param_2;
  *(uint *)(DAT_000161e4 + 0xc) = *(uint *)(DAT_000161e4 + 0xc) | 1 << (param_1 & 0xff);
  return;
}



/* ======================================================================
 * 000161e8  exc_print_dump
 * ====================================================================== */

/* exc_print_dump(msg) -- print the exception dump to the debug console.
   
   *** AUTHORITATIVE SOURCE FOR THE 0x0800 EXCEPTION PAYLOAD LAYOUT. ***
   This function names every field via its own format strings, so it settles the
   layout better than any decompile of the builder does:
   
     msg+0x04  u32 reason          "Exception Reason: %s" from a 5-entry string
                                   table at DAT_00016294 (reason < 5)
     msg+0x08  u32 reg[0]  (R0)    "R0  %08X R1  %08X R2  %08X R3  %08X"
     msg+0x0C  u32 reg[1]  (R1)
     msg+0x10  u32 reg[2]  (R2)
     msg+0x14  u32 reg[3]  (R3)
     msg+0x18..0x24   R4..R7
     msg+0x28..0x34   R8..R11
     msg+0x38  R12   msg+0x3C  SP   msg+0x40  LR   msg+0x44  PC
     msg+0x48  CPSR  msg+0x4C  SPSR
     msg+0x50  char fname[]        "Assert at File %s, Error %u"
   
   With the 4-byte WSM header stripped, that is payload+0 reason, payload+4 reg[18]
   (spanning payload+4..+0x4B), payload+0x4C fname -- **byte-exact with cw1200's
     u32 reason; u32 reg[18]; char fname[48];
   and with MsgLen 0x80 giving a 124-byte payload = 4 + 72 + 48.**  Two independent
   confirmations of the same struct.
   
   Note the console labels reg[1] as "Error" while the driver treats it as the assert
   line number.  fw_assert(file, line, code) puts file in r0, line in r1, code in r2,
   so reg[1] IS the line and the console's label is simply wrong.  assert_table.py's
   (file, line, code) recovery is consistent with the driver, not with this string.
   
   *** CORRECTION to the plate comment on exc_build_indication_and_spin
   (0x000163F0): that comment described the register block as landing at msg+2..0x27
   and fname at msg+0x28, read off a decompiled copy loop whose bounds Ghidra
   renders unreliably (the `dst` temporary is reused as both the loop limit and the
   memcpy destination).  Trust the layout above instead. *** */

void exc_print_dump(int param_1)

{
  func_0xfff014ea(s_Sent_Exception_Indication_to_hos_0001626c);
  if (*(uint *)(param_1 + 4) < 5) {
    func_0xfff014ea(s_Exception_Reason____s_00016298,
                    *(undefined4 *)(DAT_00016294 + *(uint *)(param_1 + 4) * 4));
    if (*(int *)(param_1 + 4) == 4) {
      func_0xfff014ea(s_Assert_at_File__s__Error__u_000162b0,param_1 + 0x50,
                      *(undefined4 *)(param_1 + 0xc));
    }
  }
  else {
    func_0xfff014ea(s_Unknown_Exception_Reason____08x_000162d0);
  }
  func_0xfff014ea(s_R0____08X_R1____08X_R2____08X_R3_000162f4,*(undefined4 *)(param_1 + 8),
                  *(undefined4 *)(param_1 + 0xc),*(undefined4 *)(param_1 + 0x10),
                  *(undefined4 *)(param_1 + 0x14));
  func_0xfff014ea(s_R4____08X_R5____08X_R6____08X_R7_0001632c,*(undefined4 *)(param_1 + 0x18),
                  *(undefined4 *)(param_1 + 0x1c),*(undefined4 *)(param_1 + 0x20),
                  *(undefined4 *)(param_1 + 0x24));
  func_0xfff014ea(s_R8____08X_R9____08X_R10___08X_R1_00016364,*(undefined4 *)(param_1 + 0x28),
                  *(undefined4 *)(param_1 + 0x2c),*(undefined4 *)(param_1 + 0x30),
                  *(undefined4 *)(param_1 + 0x34));
  func_0xfff014ea(s_R12___08X_SP____08X_LR____08X_PC_0001639c,*(undefined4 *)(param_1 + 0x38),
                  *(undefined4 *)(param_1 + 0x3c),*(undefined4 *)(param_1 + 0x40),
                  *(undefined4 *)(param_1 + 0x44));
  func_0xfff014ea(s_CPSR___08X_SPSR___08X_000163d4,*(undefined4 *)(param_1 + 0x48),
                  *(undefined4 *)(param_1 + 0x4c));
  return;
}



/* ======================================================================
 * 000163f0  exc_build_indication_and_spin
 * ====================================================================== */

/* exc_build_indication_and_spin(frame) -- builds the WSM exception indication and
   NEVER RETURNS.  This is the firmware side of every crash the driver reports, and
   the anchor for symbolize.py.
   
     msg = g_exc_msg;
     copy the pushed exception frame into the register block;
     if (reason == 4)  memcpy(fname_field, *(char **)&reg[0], 0x2F);  /* assert only */
     *(u16 *)(msg + 2) = 0x0800;                 /* MsgId  = exception indication */
     *(u16 *)(msg + 0) = 0x0080;                 /* MsgLen = 128 */
     hif_post_message(msg);
     mac_phy_stop_for_exception();
     exc_print_dump(msg);                        /* console dump */
     for (;;) dbg_console_poll();                /* never returns */
   
   *** FOR THE PAYLOAD LAYOUT, SEE exc_print_dump (0x000161E8), NOT THIS FUNCTION. ***
   An earlier version of this comment gave the register block as msg+2..msg+0x27 and
   fname at msg+0x28, taken from the decompiled copy loop.  Ghidra renders that loop
   unreliably -- the `dst` temporary serves as both the loop limit and the memcpy
   destination -- and the offsets it implies contradict exc_print_dump, which names
   every field explicitly via format strings.  The real layout is:
   
     msg+0x04 reason, msg+0x08..0x4C reg[0..17] (R0..R12, SP, LR, PC, CPSR, SPSR),
     msg+0x50 fname.
   
   **MsgId 0x0800, MsgLen 0x80.** 128 minus the 4-byte header is a 124-byte payload,
   matching cw1200's `u32 reason; u32 reg[18]; char fname[48]` = 4 + 72 + 48 exactly.
   
   **reason 4 == assert**, and only then is fname populated -- which is why non-assert
   dumps (undef instr / prefetch abort / data abort) carry a stale or empty fname and
   must be resolved from PC.  assert_table.py covers the first case, symbolize.py the
   second.  For asserts, reg[1] is the line number and reg[2] the code (the console's
   "Error %u" label on reg[1] is a firmware misnomer).
   
   **The firmware deliberately spins forever afterwards.** It does not reset and does
   not return to the faulting context, so the host always has time to read the
   indication out -- and the trace ring at 0xFFF0364C still holds the last 32 frame
   events (see trace_push_byte).  But the chip is dead until the driver reloads it,
   which is why a single assert ends the session rather than degrading it.
   
   Reason codes, from the four vector entries that call this:
     0 = undefined instruction (exc_vector_undef_instr,     0x000165C4)
     1 = prefetch abort        (exc_vector_prefetch_abort,  0x000165F0)
     2 = data abort            (exc_vector_data_abort,      0x000165E0)
     3 = other                 (exc_vector_reason3,         0x000165D0)
     4+ = fw_assert */

void exc_build_indication_and_spin(undefined4 *param_1)

{
  undefined2 *puVar1;
  undefined4 *dst;
  undefined4 *puVar2;
  undefined4 uVar3;
  
  puVar1 = DAT_00016438;
  puVar2 = (undefined4 *)(DAT_00016438 + 2);
  dst = (undefined4 *)(DAT_00016438 + 0x28);
  do {
    uVar3 = *param_1;
    param_1 = param_1 + 1;
    *puVar2 = uVar3;
    puVar2 = puVar2 + 1;
  } while (puVar2 < dst);
  if (*(int *)(puVar1 + 2) == 4) {
    fw_memcpy(dst,*(void **)(puVar1 + 4),0x2f);
    *(undefined1 *)((int)puVar1 + 0x7f) = 0;
  }
  puVar1[1] = 0x800;
  *puVar1 = 0x80;
  hif_post_message(puVar1);
  mac_phy_stop_for_exception();
  exc_print_dump(puVar1);
  do {
    dbg_console_poll();
  } while( true );
}



/* ======================================================================
 * 0001643c  trace_push_byte
 * ====================================================================== */

/* trace_push_byte(v) -- append to the firmware trace ring.
   
   *** THERE IS A LIVE FIRMWARE TRACE BUFFER, AND IT IS IN TCM. ***
   Base = 0xFFF0364C (DAT_00016584), i.e. inside the 0xFFF00000 TCM blob, not the
   0x0400xxxx data region.  Four independent rings share it, each with its own
   monotonically-increasing counter that is masked to index (so the counter also
   tells you how many events have ever been pushed):
   
     +0x000  u8  ring[32]    count = u16 @ +0x020   trace_push_byte  (0x0001643C)
     +0x022  u16 ring[16]    count = u16 @ +0x042   trace_push_event (0x0001644E)
     +0x044  u32 ring[32]    count = u32 @ 0xFFF03710  trace_push_pair (0x000164DC)
     +0x0C8  u32 ring[32]    count = u32 @ 0xFFF03794  trace_push_word (0x000164F8)
   
   Total span 0xFFF0364C .. 0xFFF03794, **328 bytes -- one WSM 0x0000 read**
   (that command clamps at 1024 bytes and does not validate the address, so the TCM
   range is reachable).
   
   Who writes what:
     * trace_push_event encodes frame control: values matching 5 -> OR 0x8000,
       matching 6 -> OR 0x4000, and stores the second halfword alongside.
     * trace_push_pair is called from rx_handler_main_loop with (fc, len) and from
       txp_scheduler_run / txp_pipe_tx_done_retry with per-frame values -- so the
       pair ring is effectively a TX/RX frame log.
   
   **Why this matters for the crash-log work:** after a `[BH] Fatal error` the
   firmware spins forever in dbg_console_poll rather than resetting, so this buffer
   still holds the last 32 frame events at the moment of the fault.  Reading it
   alongside the 0x0800 exception indication would give the sequence leading up to
   an assert, not just the faulting PC.  That is a concrete addition to
   symbolize.py's output and needs no firmware modification.
   
   Unverified: whether the driver's bulk-read path tolerates a TCM address in
   practice.  One DUT read settles it. */

void trace_push_byte(undefined1 param_1)

{
  int iVar1;
  
  iVar1 = DAT_00016584;
  *(undefined1 *)(DAT_00016584 + (*(ushort *)(DAT_00016584 + 0x20) & 0x1f)) = param_1;
  *(short *)(iVar1 + 0x20) = *(short *)(iVar1 + 0x20) + 1;
  return;
}



/* ======================================================================
 * 0001644e  trace_push_event
 * ====================================================================== */

void trace_push_event(ushort *param_1)

{
  ushort uVar1;
  int iVar2;
  ushort uVar3;
  
  iVar2 = DAT_00016584;
  uVar3 = *param_1;
  if ((DAT_00016588 & uVar3) == 5) {
    uVar1 = param_1[1];
    uVar3 = 0x8000;
  }
  else {
    if ((DAT_00016588 & uVar3) != 6) goto LAB_00016478;
    uVar1 = param_1[1];
    uVar3 = 0x4000;
  }
  uVar3 = uVar3 | uVar1;
LAB_00016478:
  *(ushort *)((*(ushort *)(DAT_00016584 + 0x42) & 0xf) * 2 + DAT_00016584 + 0x22) = uVar3;
  *(short *)(iVar2 + 0x42) = *(short *)(iVar2 + 0x42) + 1;
  return;
}



/* ======================================================================
 * 00016488  trace_push_event_filtered
 * ====================================================================== */

void trace_push_event_filtered(uint param_1)

{
  int iVar1;
  uint uVar2;
  
  iVar1 = DAT_00016584;
  uVar2 = DAT_00016588 & param_1;
  if (((uVar2 != DAT_0001658c) && (uVar2 != DAT_00016590)) && (uVar2 != DAT_00016590 + 0x1a)) {
    if (-1 < (int)(param_1 * 0x200000)) {
      *(short *)((*(ushort *)(DAT_00016584 + 0x42) & 0xf) * 2 + DAT_00016584 + 0x22) =
           (short)param_1;
      *(short *)(iVar1 + 0x42) = *(short *)(iVar1 + 0x42) + 1;
      return;
    }
    iVar1 = (*(ushort *)(DAT_00016584 + 0x42) - 1 & 0xf) * 2 + DAT_00016584;
    *(ushort *)(iVar1 + 0x22) = *(ushort *)(iVar1 + 0x22) | 0x400;
    *(uint *)(DAT_00016594 + 0x10) = *(uint *)(DAT_00016594 + 0x10) | 0x400;
  }
  return;
}



/* ======================================================================
 * 000164dc  trace_push_pair
 * ====================================================================== */

void trace_push_pair(uint param_1,int param_2)

{
  int iVar1;
  
  iVar1 = DAT_00016594;
  *(uint *)((*(uint *)(DAT_00016594 + -0x7c) & 0x1f) * 4 + DAT_00016584 + 0x44) =
       param_1 | param_2 << 0x10;
  *(int *)(iVar1 + -0x7c) = *(int *)(iVar1 + -0x7c) + 1;
  return;
}



/* ======================================================================
 * 000164f8  trace_push_word
 * ====================================================================== */

void trace_push_word(undefined4 param_1)

{
  int iVar1;
  
  iVar1 = DAT_00016594;
  *(undefined4 *)((*(uint *)(DAT_00016594 + 8) & 0x1f) * 4 + DAT_00016584 + 200) = param_1;
  *(int *)(iVar1 + 8) = *(int *)(iVar1 + 8) + 1;
  return;
}



/* ======================================================================
 * 000165c4  exc_vector_undef_instr
 * ====================================================================== */

void exc_vector_undef_instr
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  int in_lr;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  puStack00000014 = &uStack_20;
  iStack0000001c = in_lr + -4;
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar3 = &uStack_24;
  uStack_24 = 0;
  uVar4 = 0x16650;
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 000165d0  exc_vector_reason3
 * ====================================================================== */

void exc_vector_reason3(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 uStack_4c;
  undefined4 uStack_48;
  undefined4 uStack_44;
  undefined4 uStack_40;
  undefined4 uStack_3c;
  
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  puVar3 = &uStack_4c;
  uStack_4c = 3;
  uVar4 = 0x16650;
  uStack_48 = param_1;
  uStack_44 = param_2;
  uStack_40 = param_3;
  uStack_3c = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 000165e0  exc_vector_data_abort
 * ====================================================================== */

void exc_vector_data_abort
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  int in_lr;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 *puStack00000014;
  int iStack00000018;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  iStack00000018 = in_lr + -4;
  puStack00000014 = &uStack_20;
  iStack0000001c = in_lr + -8;
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar3 = &uStack_24;
  uStack_24 = 2;
  uVar4 = 0x16650;
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 000165f0  exc_vector_prefetch_abort
 * ====================================================================== */

void exc_vector_prefetch_abort
               (undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  int in_lr;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  undefined4 uStack_18;
  undefined4 uStack_14;
  
  puStack00000014 = &uStack_20;
  iStack0000001c = in_lr + -4;
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar3 = &uStack_24;
  uStack_24 = 1;
  uVar4 = 0x16650;
  uStack_20 = param_1;
  uStack_1c = param_2;
  uStack_18 = param_3;
  uStack_14 = param_4;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00016610  fw_assert
 * ====================================================================== */

/* fw_assert(const char *file, u32 line, u32 code)  -- firmware assert helper.
   
   Raises the exception that the host driver reports via
   wsm_handle_exception(): reason >= 4, reg[1] = line, reg[2] = code, and the
   48-byte fname field carries `file`.
   
   69 call sites across 26 source files.  Each one sets r0/r1/r2 from literal
   pools immediately before the blx, so the whole (file, line, code) table is
   statically recoverable -- see fw/assert_table.py in the research tree.
   Line numbers above 255 are built as `mov r1,#0xff; add r1,#n`.
   
   Known field crash: PC 0x0001493F / LR 0x00014943 is the blx at 0x0001493E,
   i.e. ASSERT syn_start.c:1147 code 0x2F -- WSM_START_REQ (0x0017) issued
   before the beacon-template MIB 0x1002, so the template length at +10 was 0. */

void fw_assert(char *file,uint line,uint code)

{
  int iVar1;
  int iVar2;
  undefined4 *unaff_r5;
  undefined4 *puVar3;
  undefined4 uVar4;
  bool bVar5;
  uint in_cpsr;
  undefined4 uStack_4c;
  char *pcStack_48;
  uint uStack_44;
  uint uStack_40;
  
  bVar5 = (in_cpsr >> 0x1e & 1) != 0;
  puVar3 = &uStack_4c;
  uStack_4c = 4;
  uVar4 = 0x16650;
  pcStack_48 = file;
  uStack_44 = line;
  uStack_40 = code;
  iVar2 = exc_build_indication_and_spin(puVar3);
  iVar1 = DAT_00016680;
  if (bVar5) {
    uVar4 = *unaff_r5;
    puVar3 = (undefined4 *)unaff_r5[-1];
    iVar2 = unaff_r5[-5];
  }
  puVar3[-1] = uVar4;
  puVar3[-2] = 0;
  if (iVar2 == 0) {
    uVar4 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar4 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar4;
  return;
}



/* ======================================================================
 * 00016654  fw_halt_set_ctrl
 * ====================================================================== */

void fw_halt_set_ctrl(int param_1)

{
  int iVar1;
  undefined4 uVar2;
  
  iVar1 = DAT_00016680;
  if (param_1 == 0) {
    uVar2 = 0x10;
  }
  else {
    if ((*(uint *)(DAT_00016680 + 0x24) & 1) != 0) {
      return;
    }
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    uVar2 = 1;
  }
  *(undefined4 *)(iVar1 + 0x24) = uVar2;
  return;
}



/* ======================================================================
 * 00016676  dbg_take_and_clear_reg30
 * ====================================================================== */

undefined4 dbg_take_and_clear_reg30(void)

{
  undefined4 uVar1;
  
  uVar1 = *(undefined4 *)(DAT_00016684 + 0x30);
  *(undefined4 *)(DAT_00016684 + 0x30) = 0;
  return uVar1;
}



/* ======================================================================
 * 00016688  dbg_set_period
 * ====================================================================== */

void dbg_set_period(int param_1)

{
  if (param_1 == 0) {
    param_1 = 1;
  }
  *(int *)(DAT_00016694 + 4) = param_1;
  return;
}



/* ======================================================================
 * 00016698  sched_register_task
 * ====================================================================== */

void sched_register_task(undefined4 param_1,undefined4 param_2)

{
  int iVar1;
  
  iVar1 = fw_clz();
  *(undefined4 *)(DAT_000166a8 + iVar1 * 4) = param_2;
  return;
}



/* ======================================================================
 * 000166ac  timer_entry_init
 * ====================================================================== */

void timer_entry_init(int param_1,undefined4 param_2,undefined4 param_3)

{
  *(undefined4 *)(param_1 + 0xc) = param_2;
  *(undefined4 *)(param_1 + 0x10) = param_3;
  *(undefined4 *)(param_1 + 4) = 0;
  return;
}



/* ======================================================================
 * 000166b8  fw_halt_clear
 * ====================================================================== */

void fw_halt_clear(void)

{
  fw_halt_set_ctrl(0);
  return;
}



/* ======================================================================
 * 000166c2  fw_halt_and_dump_phy
 * ====================================================================== */

void fw_halt_and_dump_phy(void)

{
  fw_halt_set_ctrl(1);
  if (*DAT_000168c8 == '\x02') {
    rf_init_stage_d();
    rf_init_stage_a();
    rf_init_stage_b();
    rf_init_stage_c();
  }
  return;
}



/* ======================================================================
 * 000166e4  dbg_expand_byte_table
 * ====================================================================== */

void dbg_expand_byte_table(void)

{
  undefined4 uVar1;
  uint uVar2;
  uint *puVar3;
  byte *pbVar4;
  
  uVar1 = DAT_000168d4;
  uVar2 = 0;
  puVar3 = DAT_000168cc;
  pbVar4 = DAT_000168d0;
  do {
    *puVar3 = (uint)*pbVar4;
    puVar3 = puVar3 + 1;
    pbVar4 = pbVar4 + 1;
    uVar2 = uVar2 + 1;
  } while (uVar2 < 0x20);
  reg_write_list_apply(uVar1);
  return;
}



/* ======================================================================
 * 00016702  phy_mode_apply
 * ====================================================================== */

void phy_mode_apply(int param_1)

{
  undefined4 uVar1;
  
  if (param_1 != 1) {
    if ((param_1 != 2) && (param_1 != 3)) {
      if (param_1 != 4) {
        return;
      }
      rf_cal_if_phy_ready(1,1);
      uVar1 = 5;
      goto LAB_0001671e;
    }
    rf_cal_if_phy_ready(1,1);
  }
  uVar1 = 0;
LAB_0001671e:
  phy_notify_mode_change(uVar1);
  return;
}



/* ======================================================================
 * 00016732  phy_copy_cfg_slot
 * ====================================================================== */

void phy_copy_cfg_slot(int param_1)

{
  *(undefined4 *)(param_1 * 4 + DAT_000168c8 + 0x60) = *(undefined4 *)(DAT_000168c8 + 0x4c);
  return;
}



/* ======================================================================
 * 0001673e  phy_set_channel_full
 * ====================================================================== */

void phy_set_channel_full(ushort param_1)

{
  char cVar1;
  int iVar2;
  int iVar3;
  undefined2 *puVar4;
  undefined2 uVar5;
  uint uVar6;
  undefined4 uVar7;
  
  iVar2 = DAT_000168c8;
  *(ushort *)(DAT_000168c8 + 6) = param_1;
  phy_apply_cfg_pair();
  if (*(char *)(iVar2 + 0x11) != '\0') {
    phy_mode_apply(3);
  }
  phy_program_clock_divisor();
  rf_measure_temp_and_vbat(0);
  phy_copy_cfg_slot(0);
  if (*(char *)(iVar2 + 2) == '\0') {
    cVar1 = *(char *)(iVar2 + 0xd);
  }
  else {
    if (*(char *)(iVar2 + 2) != '\x01') goto LAB_000167c0;
    cVar1 = *(char *)(iVar2 + 0x15);
  }
  if (cVar1 == '\0') {
    uVar7 = 0;
    phy_copy_cfg_slot(1);
    iVar3 = DAT_000168dc;
    if (*(char *)(iVar2 + 2) == '\0') {
      uVar7 = DAT_000168d8;
    }
    uVar6 = 0;
    do {
      *(undefined4 *)(uVar6 * 4 + iVar3 + 0x28) = uVar7;
      uVar6 = uVar6 + 1;
    } while (uVar6 < 0x10);
    phy_mode_apply(2);
    if (*(char *)(iVar2 + 2) == '\0') {
      if (*(char *)(iVar2 + 0xd) == '\x01') {
        *(ushort *)(iVar2 + 0x16) = param_1;
      }
    }
    else if ((*(char *)(iVar2 + 2) == '\x01') && (*(char *)(iVar2 + 0x15) == '\x01')) {
      *(ushort *)(DAT_000168c8 + 0x82) = param_1;
    }
  }
LAB_000167c0:
  phy_agc_enable(1);
  uVar5 = phy_lookup_by_threshold();
  puVar4 = DAT_000168e0;
  DAT_000168e0[0x18] = uVar5;
  uVar5 = phy_txpower_from_rate_table(param_1 & 0xff,0);
  *puVar4 = uVar5;
  uVar5 = phy_txpower_from_rate_table(param_1 & 0xff,1);
  puVar4[1] = uVar5;
  phy_set_freq_offset();
  return;
}



/* ======================================================================
 * 0001683a  rf_reg_op_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x0001684a) */
/* WARNING: Removing unreachable block (ram,0x0001684a) */
/* WARNING: Removing unreachable block (ram,0x0001687a) */

undefined4 rf_reg_op_dispatch(uint param_1)

{
  undefined4 uVar1;
  uint uVar2;
  
  uVar2 = (int)(param_1 & 0xf00) >> 8;
                    /* WARNING: Could not recover jumptable at 0x0001684a. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0001684e <= uVar2) {
    uVar2 = (uint)DAT_0001684e;
  }
  uVar1 = (*(code *)((uint)*(byte *)(uVar2 + 0x1684f) * 2 + 0x1684f))();
  return uVar1;
}



/* ======================================================================
 * 0001687e  rf_settle_time_for_op
 * ====================================================================== */

int rf_settle_time_for_op(uint param_1)

{
  int iVar1;
  
  if (*(char *)(DAT_000168c8 + 2) == '\x01') {
    iVar1 = rf_reg_op_dispatch(param_1);
    iVar1 = (param_1 & 0xff) * 5 + iVar1;
  }
  else if (param_1 < 0xe) {
    iVar1 = param_1 * 5 + DAT_000168f8;
  }
  else {
    iVar1 = DAT_000168f8 + 0x4d;
  }
  return iVar1 * 1000;
}



/* ======================================================================
 * 000168b6  phy_write_reg_if_mode2
 * ====================================================================== */

void phy_write_reg_if_mode2(void)

{
  if (*DAT_000168c8 == '\x02') {
    phy_write_reg_neg64();
  }
  return;
}



/* ======================================================================
 * 000168fc  rf_cal_if_phy_ready
 * ====================================================================== */

void rf_cal_if_phy_ready(void)

{
  if (*DAT_00016970 == '\x02') {
    rf_calibrate_iq_dc();
  }
  return;
}



/* ======================================================================
 * 0001690c  rf_cal_gate
 * ====================================================================== */

void rf_cal_gate(int param_1)

{
  char cVar1;
  uint local_c;
  
  local_c = 0;
  cVar1 = *DAT_00016970;
  if (cVar1 == '\x02') {
    local_c = *(uint *)(*(int *)(DAT_00016970 + 0x38) + DAT_00016974 + 4);
  }
  if (param_1 == 0) {
    local_c = local_c & 0xfffb7fff;
  }
  else {
    local_c = local_c | 0x48000;
  }
  if ((cVar1 == '\x01') || (cVar1 == '\x02')) {
    *(uint *)(*(int *)(DAT_00016970 + 0x38) + DAT_00016974 + 4) = local_c;
  }
  return;
}



/* ======================================================================
 * 0001694e  rf_cal_save_refs
 * ====================================================================== */

void rf_cal_save_refs(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  undefined4 *puVar1;
  
  puVar1 = DAT_00016978;
  *DAT_00016978 = param_1;
  puVar1[1] = param_2;
  puVar1[2] = param_3;
  puVar1[3] = param_4;
  return;
}



/* ======================================================================
 * 00016956  rf_cal_get_ref0
 * ====================================================================== */

undefined4 rf_cal_get_ref0(void)

{
  return *DAT_00016978;
}



/* ======================================================================
 * 0001695c  rf_cal_get_ref1
 * ====================================================================== */

undefined4 rf_cal_get_ref1(void)

{
  return *(undefined4 *)(DAT_00016978 + 4);
}



/* ======================================================================
 * 00016962  rf_cal_get_ref2
 * ====================================================================== */

undefined4 rf_cal_get_ref2(void)

{
  return *(undefined4 *)(DAT_00016978 + 8);
}



/* ======================================================================
 * 00016968  rf_cal_get_ref3
 * ====================================================================== */

undefined4 rf_cal_get_ref3(void)

{
  return *(undefined4 *)(DAT_00016978 + 0xc);
}



/* ======================================================================
 * 0001697c  phy_notify_mode_change
 * ====================================================================== */

void phy_notify_mode_change(void)

{
  if ((*DAT_00016a44 != '\x01') && (*DAT_00016a44 == '\x02')) {
    rf_reload_iq_regs();
  }
  return;
}



/* ======================================================================
 * 00016990  phy_cal_cmd_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x000169a0) */
/* WARNING: Removing unreachable block (ram,0x000169a0) */

void phy_cal_cmd_dispatch(byte *param_1)

{
  uint uVar1;
  
  uVar1 = (uint)*param_1;
                    /* WARNING: Could not recover jumptable at 0x000169a0. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_000169a4 <= uVar1) {
    uVar1 = (uint)DAT_000169a4;
  }
  (*(code *)((uint)*(byte *)(uVar1 + 0x169a5) * 2 + 0x169a5))
            (param_1,DAT_00016a48 + -8,DAT_00016a48 + -4);
  return;
}



/* ======================================================================
 * 00016a16  phy_cal_cmd_start
 * ====================================================================== */

void phy_cal_cmd_start(undefined4 param_1)

{
  undefined1 local_18 [4];
  undefined4 local_14;
  undefined4 local_c;
  
  local_18[0] = 1;
  local_c = 8;
  local_14 = param_1;
  phy_cal_cmd_dispatch(local_18);
  return;
}



/* ======================================================================
 * 00016a30  phy_cal_cmd_stop
 * ====================================================================== */

void phy_cal_cmd_stop(void)

{
  undefined1 local_18 [24];
  
  local_18[0] = 0;
  phy_cal_cmd_dispatch(local_18);
  return;
}



/* ======================================================================
 * 00016b4e  phy_cal_step_done
 * ====================================================================== */

void phy_cal_step_done(void)

{
  fw_halt_clear();
  *(undefined1 *)(DAT_00016d48 + -0x2d) = 4;
  return;
}



/* ======================================================================
 * 00016b5e  phy_cal_arm_timer
 * ====================================================================== */

void phy_cal_arm_timer(uint param_1)

{
  int iVar1;
  
  iVar1 = DAT_00016d48;
  if ((*(ushort *)(DAT_00016d48 + -0x3a) != param_1) || (*(char *)(DAT_00016d48 + 0x44) != '\0')) {
    phy_set_channel_full();
    *(undefined1 *)(iVar1 + 0x44) = 0;
  }
  *(undefined1 *)(iVar1 + -0x2d) = 3;
  return;
}



/* ======================================================================
 * 00016b82  phy_cal_set_flag
 * ====================================================================== */

void phy_cal_set_flag(uint param_1,int param_2)

{
  char cVar1;
  int iVar2;
  undefined1 uVar3;
  
  iVar2 = DAT_00016d48;
  if ((*(byte *)(DAT_00016d48 + -0x3d) == param_1) && (param_2 == 0)) {
    if (*(char *)(DAT_00016d48 + -0x3e) != '\0') {
      return;
    }
    *(undefined1 *)(DAT_00016d48 + 0x44) = 0;
    return;
  }
  if ((((param_1 == 0) || (param_1 == 1)) || (param_1 == 2)) ||
     ((param_1 != 3 || (-1 < (int)((uint)*(ushort *)(DAT_00016d60 + 0x12) << 0x1e))))) {
    uVar3 = 0;
  }
  else {
    *(undefined1 *)(DAT_00016d48 + -0x3d) = 3;
    uVar3 = 1;
  }
  *(undefined1 *)(iVar2 + -0x3e) = uVar3;
  fw_halt_and_dump_phy();
  phy_init_once();
  phy_band_cmd_dispatch(param_1);
  *(char *)(iVar2 + -0x3d) = (char)param_1;
  *(undefined1 *)(iVar2 + 0x44) = 1;
  if (*(char *)(iVar2 + -0x3e) == '\0') {
    cVar1 = *(char *)(iVar2 + -0x33);
  }
  else {
    if (*(char *)(iVar2 + -0x3e) != '\x01') goto LAB_00016bea;
    cVar1 = *(char *)(iVar2 + -0x2b);
  }
  if (cVar1 != '\0') {
    phy_mode_apply(4);
  }
LAB_00016bea:
  phy_select_rate_tables();
  return;
}



/* ======================================================================
 * 00016bfa  phy_cal_advance_stage
 * ====================================================================== */

void phy_cal_advance_stage(void)

{
  int iVar1;
  
  phy_apply_reg_init_lists();
  iVar1 = DAT_00016d48;
  if (*(char *)(DAT_00016d48 + 0x45) == '\0') {
    phy_band_cmd_dispatch(*(undefined1 *)(DAT_00016d48 + -0x3d));
    if (*(char *)(iVar1 + -0x3d) == '\x03') {
      *(undefined1 *)(iVar1 + -0x2b) = 0;
    }
    phy_cal_set_flag(*(char *)(iVar1 + -0x3d),0);
    *(undefined1 *)(iVar1 + 0x44) = 1;
    phy_cal_arm_timer(*(undefined2 *)(iVar1 + -0x3a));
  }
  else {
    *(undefined1 *)(DAT_00016d48 + 0x7c) = 1;
    phy_band_cmd_dispatch(*(undefined1 *)(iVar1 + -0x3c));
    if (*(char *)(iVar1 + -0x3d) == '\x03') {
      *(undefined1 *)(iVar1 + -0x2b) = 0;
    }
    phy_cal_set_flag(*(undefined1 *)(iVar1 + -0x3c),0);
    *(undefined1 *)(iVar1 + 0x44) = 1;
    phy_cal_arm_timer(*(undefined2 *)(iVar1 + -0x38));
    *(undefined1 *)(iVar1 + 0x45) = 0;
  }
  *(undefined1 *)(iVar1 + -0x2c) = 2;
  return;
}



/* ======================================================================
 * 00016c5c  phy_cal_step_next
 * ====================================================================== */

void phy_cal_step_next(void)

{
  fw_halt_and_dump_phy();
  phy_cal_advance_stage();
  *(undefined1 *)(DAT_00016d48 + -0x2d) = 3;
  return;
}



/* ======================================================================
 * 00016c70  phy_cal_step_start
 * ====================================================================== */

void phy_cal_step_start(void)

{
  int iVar1;
  
  fw_halt_clear();
  iVar1 = DAT_00016d48;
  *(undefined1 *)(DAT_00016d48 + -0x2d) = 1;
  if (*(char *)(iVar1 + -0x3d) == '\x03') {
    *(undefined1 *)(iVar1 + -0x2b) = 0;
    *(undefined2 *)(iVar1 + 0x42) = 100;
    *(undefined1 *)(iVar1 + 0x44) = 1;
  }
  return;
}



/* ======================================================================
 * 00016c92  phy_cal_step_measure
 * ====================================================================== */

void phy_cal_step_measure(void)

{
  int iVar1;
  
  fw_halt_and_dump_phy();
  iVar1 = DAT_00016d48;
  phy_cal_apply_substate(*(undefined1 *)(DAT_00016d48 + -0x3d));
  dbg_expand_byte_table();
  if (*(char *)(iVar1 + -0x2b) == '\0') {
    phy_cal_set_flag(3,1);
    phy_cal_arm_timer(*(undefined2 *)(iVar1 + 0x42));
  }
  if (*(char *)(iVar1 + -0x33) == '\0') {
    phy_cal_set_flag(2,1);
    phy_cal_arm_timer(*(undefined2 *)(iVar1 + -0x2a));
  }
  if ((*(char *)(iVar1 + -0x33) == '\x01') && (*(char *)(iVar1 + -0x2b) == '\x01')) {
    *(undefined1 *)(iVar1 + -0x3d) = 4;
  }
  *(undefined1 *)(iVar1 + -0x2d) = 2;
  return;
}



/* ======================================================================
 * 00016d8c  thunk_16c92
 * ====================================================================== */

void thunk_16c92(void)

{
  phy_cal_step_measure();
  return;
}



/* ======================================================================
 * 00016d94  thunk_16c70
 * ====================================================================== */

void thunk_16c70(void)

{
  phy_cal_step_start();
  return;
}



/* ======================================================================
 * 00016d9c  thunk_16b4e
 * ====================================================================== */

void thunk_16b4e(void)

{
  phy_cal_step_done();
  return;
}



/* ======================================================================
 * 00016dac  phy_cal_run_step_timed
 * ====================================================================== */

undefined8 phy_cal_run_step_timed(int param_1,int param_2,undefined4 *param_3)

{
  int iVar1;
  undefined4 uVar2;
  int iVar3;
  
  iVar3 = DAT_0001702c;
  iVar1 = DAT_00017028;
  if (param_1 == 0) {
    *(undefined1 *)(DAT_0001702c + 0x14) = 0;
    if (param_2 != 0) {
      *(char *)(iVar1 + 0x5f) = (char)param_2;
    }
    phy_cal_step_next();
    if (param_2 != 0) {
      *(undefined1 *)(iVar1 + 0x5f) = 0;
    }
    *(undefined1 *)(iVar3 + 0x14) = 1;
    uVar2 = fw_read_timer();
    *(undefined4 *)(DAT_0001702c + 0xc) = uVar2;
    *param_3 = 0x78;
    return 1;
  }
  if (param_1 == 1) {
    iVar3 = fw_read_timer();
    iVar1 = DAT_0001702c;
    if (iVar3 - *(int *)(DAT_0001702c + 0xc) < 0) {
      iVar3 = fw_read_timer();
      iVar3 = *(int *)(iVar1 + 0xc) - iVar3;
    }
    else {
      iVar3 = fw_read_timer();
      iVar3 = iVar3 - *(int *)(iVar1 + 0xc);
    }
    if (0 < 0x78 - iVar3) {
      fw_delay_loop();
    }
  }
  return CONCAT44(param_1,2);
}



/* ======================================================================
 * 00016e2a  phy_cal_set_channel_and_arm
 * ====================================================================== */

void phy_cal_set_channel_and_arm(int param_1,uint param_2,int param_3)

{
  int iVar1;
  
  iVar1 = DAT_00017028;
  if (((param_1 == 3) && (param_3 == 0)) &&
     ((*(char *)(DAT_00017028 + 0x15) != '\x01' || (*(ushort *)(DAT_00017028 + 0x82) != param_2))))
  {
    *(undefined1 *)(DAT_00017028 + 0x15) = 0;
    *(short *)(iVar1 + 0x82) = (short)param_2;
    *(undefined1 *)(iVar1 + 0x84) = 1;
  }
  phy_cal_set_flag(param_1,0);
  *(undefined1 *)(DAT_0001702c + 0x14) = 1;
  phy_cal_arm_timer(param_2);
  return;
}



/* ======================================================================
 * 00016e6a  phy_temp_compensate
 * ====================================================================== */

void phy_temp_compensate(undefined4 param_1,undefined4 param_2,int param_3)

{
  short sVar1;
  undefined1 auStack_18 [4];
  undefined1 auStack_14 [4];
  undefined1 auStack_10 [4];
  
  if (1 < *(byte *)(DAT_00017028 + 0x13)) {
    sVar1 = fw_div_scaled(param_3 << 4,10);
    phy_program_gain_for_channel((int)sVar1,param_1,param_2,auStack_10,auStack_14,auStack_18);
  }
  return;
}



/* ======================================================================
 * 00016e9c  phy_get_tx_power_range
 * ====================================================================== */

void phy_get_tx_power_range
               (undefined2 *param_1,undefined2 *param_2,undefined2 *param_3,undefined2 *param_4)

{
  int iVar1;
  
  *param_1 = 0xff60;
  iVar1 = DAT_00017030;
  *param_3 = 0xff60;
  *param_2 = *(undefined2 *)(iVar1 + 0x14);
  *param_4 = *(undefined2 *)(DAT_00017030 + 0xa6);
  return;
}



/* ======================================================================
 * 00016f6c  phy_state_cmd_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00016f7e) */
/* WARNING: Removing unreachable block (ram,0x00016f7e) */

void phy_state_cmd_dispatch(byte *param_1,undefined4 param_2,undefined4 param_3)

{
  uint uVar1;
  int iVar2;
  
  uVar1 = (uint)*param_1;
                    /* WARNING: Could not recover jumptable at 0x00016f7e. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_00016f82 <= uVar1) {
    uVar1 = (uint)DAT_00016f82;
  }
  iVar2 = (uint)*(byte *)(uVar1 + 0x16f83) * 2;
  (*(code *)(iVar2 + 0x16f83))
            (param_1,*(undefined1 *)(DAT_00017028 + 0x5d),param_3,iVar2,DAT_00017034);
  return;
}



/* ======================================================================
 * 0001704a  reg_write_list_apply
 * ====================================================================== */

undefined4 reg_write_list_apply(int *param_1)

{
  for (; (int *)*param_1 != (int *)0xffffffff; param_1 = param_1 + 2) {
    *(int *)*param_1 = param_1[1];
  }
  return 0;
}



/* ======================================================================
 * 00017052  reg_write_list_apply
 * ====================================================================== */

/* reg_write_list_apply(list) -- interpret a {address, value} register-write list.
   
     for (p = list; *p != 0xFFFFFFFF; p += 2)
         *(u32 *)p[0] = p[1];
     return 0;
   
   Terminator is a **0xFFFFFFFF address**, not a null.
   
   The four lists applied by phy_apply_reg_init_lists (0x000171FA) live in .data and
   are verified from the carved image:
   
     0x04000C10  0AB80108 = 00200300
     0x04000C20  0AB8807C = 00000001, 0AB88058 = 000063D9,
                 0AB8808C = 0000103F, 0AB88090 = 1010103F
     0x04000C48  0AB90000 = 00000000, 0AB90014 = 0FFFFFFF
     0x04000C60  0ABA0004 = 00000033, 0ABA0008 = 0000013E, 0ABA000C = 000000A6,
                 0ABA0010 = 00000001, 0ABA0014 = 00000040, 0AB80C2C = 0000004F,
                 0ABA8540 = 0000017F, ...
   
   Target blocks are 0x0AB8xxxx / 0x0AB9xxxx / 0x0ABAxxxx, all already in
   FIRMWARE-RE.md's peripheral map.  (An earlier draft of this comment claimed these
   blocks were undocumented -- they are not; the map lists 0x0AB90000/0x0ABA0000/
   0x0ABB0000 as further PHY/RF register blocks, and records that the container's
   41 pairs span the same three.)
   
   Note on the XR01 container: its `type 2` descriptor uses the **same
   {u32 addr, u32 value} encoding**, and carve.py already decodes it into
   07-regs.txt.  But that is a *different mechanism* -- the container list is applied
   by the boot downloader at load time, before the firmware runs, whereas this
   function applies lists held in the firmware's own .data at runtime, selected by
   band and calibration state.  Same format, two independent users; do not conflate
   them.
   
   Useful consequence: because both are plain {addr, value} tables, a build-to-build
   diff of PHY register initialisation can be done on data alone -- carve.py for the
   container part, and a dump of 0x04000C10..0x04000CFF for the runtime part. */

undefined4 reg_write_list_apply(int *param_1)

{
  for (; (int *)*param_1 != (int *)0xffffffff; param_1 = param_1 + 2) {
    *(int *)*param_1 = param_1[1];
  }
  return 0;
}



/* ======================================================================
 * 0001705c  phy_build_gain_tables
 * ====================================================================== */

void phy_build_gain_tables(void)

{
  char cVar1;
  short sVar2;
  uint *puVar3;
  uint uVar4;
  int iVar5;
  undefined4 uVar6;
  int iVar7;
  int iVar8;
  int iVar9;
  int iVar10;
  short sVar11;
  short sVar12;
  char local_3ec [320];
  uint local_2ac [80];
  uint local_16c [80];
  uint *local_2c;
  uint *local_28;
  uint *local_24;
  
  iVar5 = DAT_0001739c;
  local_24 = DAT_00017390;
  local_2c = DAT_00017390 + 4;
  local_28 = DAT_00017394;
  cVar1 = *(char *)(DAT_00017398 + 2);
  iVar7 = 0;
  do {
    if (cVar1 == '\x01') {
      sVar11 = *(short *)(DAT_000173a0 + 0xda);
      iVar8 = DAT_000173a4;
    }
    else {
      sVar11 = *(short *)(DAT_000173a0 + 0x48);
      iVar8 = DAT_000173a8;
    }
    iVar9 = iVar7 * 6;
    *(undefined1 *)(iVar5 + iVar9) = *(undefined1 *)(iVar8 + iVar9);
    *(undefined2 *)(iVar9 + iVar5 + 2) = *(undefined2 *)(iVar9 + iVar8 + 2);
    *(undefined2 *)(iVar9 + iVar5 + 4) = *(undefined2 *)(iVar9 + iVar8 + 4);
    iVar8 = iVar7 * 6;
    iVar7 = iVar7 + 1;
    iVar8 = iVar8 + iVar5;
    *(short *)(iVar8 + 2) =
         (short)(((int)(((uint)*(ushort *)(iVar8 + 2) - (int)sVar11) * 0x10000) >> 0x10) + 8 >> 4);
    *(short *)(iVar8 + 4) =
         (short)(((int)(((uint)*(ushort *)(iVar8 + 4) + (int)sVar11) * 0x10000) >> 0x10) + 8 >> 4);
    puVar3 = DAT_00017394;
  } while (iVar7 < 0x16);
  iVar5 = 0;
  do {
    iVar8 = iVar5 * 4;
    local_16c[iVar5] = 0;
    local_2ac[iVar5] = 0;
    sVar11 = -0x18;
    iVar7 = iVar5 + -0x5a;
    uVar4 = 0x15;
    sVar12 = sVar11;
    do {
      iVar9 = uVar4 * 6 + DAT_0001739c;
      iVar10 = *(short *)(iVar9 + 2) + 0x18;
      if ((-iVar7 != iVar10 && iVar7 <= -iVar10) && (sVar2 = *(short *)(iVar9 + 4), sVar2 < sVar11))
      {
        local_16c[iVar5] = uVar4;
        sVar11 = sVar2;
      }
      if ((-iVar7 != iVar10 && iVar7 <= -iVar10) && (sVar2 = *(short *)(iVar9 + 4), sVar2 < sVar12))
      {
        local_2ac[iVar5] = uVar4;
        sVar12 = sVar2;
      }
      uVar4 = uVar4 - 1;
    } while (uVar4 < 0x80000000);
    uVar4 = local_16c[iVar5];
    local_3ec[iVar8] = *(char *)(DAT_0001739c + uVar4 * 6);
    iVar7 = iVar5 + 1;
    local_3ec[iVar8 + 1] = (char)*(undefined2 *)(uVar4 * 6 + DAT_0001739c + 2);
    uVar4 = local_2ac[iVar5];
    local_3ec[iVar8 + 2] = *(char *)(DAT_0001739c + uVar4 * 6);
    local_3ec[iVar8 + 3] = (char)*(undefined2 *)(uVar4 * 6 + DAT_0001739c + 2);
    iVar5 = iVar7;
  } while (iVar7 < 0x50);
  iVar5 = 0;
  do {
    iVar7 = iVar5 * 4;
    puVar3[iVar5] =
         ((byte)local_3ec[iVar7 + 1] & 0x7f) << 8 | (int)local_3ec[iVar7 + 2] << 0x10 |
         (int)local_3ec[iVar7] | ((byte)local_3ec[iVar7 + 3] & 0x7f) << 0x18;
    iVar5 = iVar5 + 1;
  } while (iVar5 < 0x50);
  uVar6 = DAT_000173b0;
  if (*(char *)(DAT_00017398 + 2) == '\x01') {
    uVar6 = DAT_000173ac;
  }
  reg_write_list_apply(uVar6);
  *local_24 = *local_24 & 0xffff80ff | *local_28 & 0x7f00;
  uVar4 = *local_2c;
  iVar5 = 0;
  do {
    if ((puVar3[iVar5] & 0x7fff) >> 8 < 0xb) {
      uVar4 = uVar4 & 0xffffff80 | iVar5 + 1U & 0x7f;
      break;
    }
    iVar5 = iVar5 + 1;
  } while (iVar5 < 0x50);
  *local_2c = uVar4;
  return;
}



/* ======================================================================
 * 000171fa  phy_apply_reg_init_lists
 * ====================================================================== */

void phy_apply_reg_init_lists(void)

{
  if ((*DAT_00017398 == '\x01') || (*DAT_00017398 == '\x02')) {
    reg_write_list_apply(DAT_000173b4);
    reg_write_list_apply(DAT_000173b8);
    reg_write_list_apply(DAT_000173bc);
    reg_write_list_apply(DAT_000173c0);
  }
  return;
}



/* ======================================================================
 * 00017222  phy_init_once
 * ====================================================================== */

void phy_init_once(void)

{
  char cVar1;
  char *pcVar2;
  int iVar3;
  uint uVar4;
  undefined4 *puVar5;
  undefined4 uVar6;
  
  uVar6 = DAT_000173c8;
  puVar5 = DAT_000173c4;
  pcVar2 = DAT_00017398;
  if (DAT_00017398[0x14] == '\0') {
    DAT_00017398[0x14] = '\x02';
    uVar4 = 0;
    do {
      *puVar5 = uVar6;
      puVar5 = puVar5 + 1;
      uVar4 = uVar4 + 1;
    } while (uVar4 < 0x41);
    cVar1 = *pcVar2;
    if ((cVar1 == '\x01') || (cVar1 == '\x02')) {
      *(undefined4 *)(DAT_000173d0 + 0x1c) = DAT_000173cc;
      iVar3 = DAT_000173d8;
      uVar6 = DAT_000173e0;
      if (*DAT_000173d4 == '\x02') {
        uVar6 = DAT_000173dc;
      }
      *(undefined4 *)(DAT_000173d8 + 8) = uVar6;
      *(undefined4 *)(iVar3 + 0x3c) = DAT_000173e4;
      *(undefined4 *)(DAT_000173ec + 0x24) = DAT_000173e8;
    }
    phy_apply_reg_init_lists();
  }
  phy_build_gain_tables(pcVar2[2]);
  return;
}



/* ======================================================================
 * 00017278  phy_program_clock_divisor
 * ====================================================================== */

void phy_program_clock_divisor(void)

{
  undefined4 uVar1;
  int iVar2;
  
  if (*(char *)(DAT_00017398 + 2) == '\0') {
    iVar2 = 0x14;
  }
  else {
    iVar2 = 10;
  }
  uVar1 = __udivsi3(*(undefined4 *)(DAT_00017398 + 0x28),1000);
  iVar2 = __udivsi3(iVar2 << 0xe,uVar1);
  *(int *)(DAT_000173f0 + 0x20) = -iVar2;
  return;
}



/* ======================================================================
 * 000172a2  phy_set_bandwidth_mode
 * ====================================================================== */

void phy_set_bandwidth_mode(int param_1)

{
  uint *puVar1;
  uint uVar2;
  uint uVar3;
  undefined4 uVar4;
  
  uVar2 = DAT_000173f4;
  puVar1 = DAT_000173d8;
  uVar4 = 0;
  uVar3 = *DAT_000173d8;
  if (param_1 == 0) {
    *DAT_000173d0 = *DAT_000173d0 & 0xfffdffff;
    uVar3 = (uVar3 & uVar2) + 0xb40000;
    uVar4 = 6;
  }
  else if (param_1 == 1) {
    *DAT_000173d0 = *DAT_000173d0 | 0x20000;
    uVar3 = (uVar3 & uVar2) + 0x1900000;
    uVar4 = 5;
  }
  else if (param_1 == 2) {
    *DAT_000173d0 = *DAT_000173d0 | 0x20000;
    uVar3 = (uVar3 & uVar2) + 0xb40000;
    uVar4 = 4;
  }
  else if (param_1 == 3) {
    *DAT_000173d0 = *DAT_000173d0 & 0xfffdffff;
    uVar3 = (uVar3 & uVar2) + 0xb40000;
    uVar4 = 7;
  }
  *puVar1 = uVar3;
  *(undefined4 *)(DAT_000173f8 + 4) = uVar4;
  return;
}



/* ======================================================================
 * 0001730c  phy_set_antenna_mode
 * ====================================================================== */

void phy_set_antenna_mode(int param_1)

{
  uint uVar1;
  
  uVar1 = *DAT_000173d8;
  if (param_1 == 0) {
    uVar1 = uVar1 & 0xffffffef;
  }
  else {
    if (param_1 != 1) {
      if (param_1 == 2) {
        uVar1 = uVar1 & 0xffffffef | 8;
      }
      else if (param_1 == 3) {
        uVar1 = uVar1 | 0x18;
      }
      goto LAB_0001732e;
    }
    uVar1 = uVar1 | 0x10;
  }
  uVar1 = uVar1 & 0xfffffff7;
LAB_0001732e:
  *DAT_000173d8 = uVar1;
  return;
}



/* ======================================================================
 * 0001734e  phy_watchdog_check
 * ====================================================================== */

void phy_watchdog_check(void)

{
  int iVar1;
  uint *puVar2;
  uint uVar3;
  
  puVar2 = DAT_00017400;
  if (((*DAT_000173fc & 1) != 0) && (-1 < *(int *)(DAT_000173d8 + 0x28) << 9)) {
    uVar3 = *DAT_00017400;
    *DAT_00017400 = uVar3 + 1;
    if (uVar3 + 1 < 3) {
      return;
    }
    func_0xfff019c8(10,0);
    iVar1 = DAT_000173f8;
    uVar3 = *(uint *)(DAT_000173f8 + 8);
    *(uint *)(DAT_000173f8 + 8) = uVar3 & 0xfffffffe;
    *(uint *)(iVar1 + 8) = uVar3 | 1;
  }
  *puVar2 = 0;
  return;
}



/* ======================================================================
 * 00017404  phy_cal_apply_substate
 * ====================================================================== */

void phy_cal_apply_substate(void)

{
  if (*DAT_000174a8 == '\x02') {
    reg_write_list_apply(DAT_000174ac);
  }
  return;
}



/* ======================================================================
 * 00017416  phy_band_cmd_dispatch
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00017438) */
/* WARNING: Removing unreachable block (ram,0x00017438) */

void phy_band_cmd_dispatch(uint param_1)

{
                    /* WARNING: Could not recover jumptable at 0x00017438. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_0001743c <= param_1) {
    param_1 = (uint)DAT_0001743c;
  }
  (*(code *)((uint)*(byte *)(param_1 + 0x1743d) * 2 + 0x1743d))
            (2,(*(uint *)(DAT_000174b0 + 4) & 0xfffffffe | 0x20000000) & DAT_000174b4 | 0x800,4);
  return;
}



/* ======================================================================
 * 000174b8  tlv_dispatch_table
 * ====================================================================== */

undefined4 tlv_dispatch_table(char *param_1,char *param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  char *pcVar2;
  
  do {
    if ((*param_1 == -0x40) || (*param_1 != -0x3f)) {
      uVar1 = (uint)(byte)param_1[1];
      param_1 = param_1 + 2;
    }
    else {
      if (param_1[2] == -1) {
        return 0;
      }
      for (pcVar2 = param_2; *(int *)(pcVar2 + 4) != 0; pcVar2 = pcVar2 + 8) {
        if (*pcVar2 == param_1[2]) {
          (**(code **)(pcVar2 + 4))
                    (param_1 + 8,*(ushort *)(param_1 + 6) - 8,*(code **)(pcVar2 + 4),*pcVar2,param_4
                    );
          break;
        }
      }
      uVar1 = (uint)*(ushort *)(param_1 + 6);
    }
    param_1 = param_1 + uVar1;
  } while( true );
}



/* ======================================================================
 * 00017520  tlv_dispatch_default
 * ====================================================================== */

void tlv_dispatch_default(undefined4 param_1)

{
  tlv_dispatch_table(param_1,DAT_0001752c);
  return;
}



/* ======================================================================
 * 000178d8  rf_set_test_tone
 * ====================================================================== */

void rf_set_test_tone(uint param_1)

{
  uint uVar1;
  
  uVar1 = 0;
  if (-1 < (int)param_1) {
    uVar1 = param_1 & 0x3f | 0x40;
  }
  *(uint *)(DAT_00017cd8 + 0x24) = uVar1;
  return;
}



/* ======================================================================
 * 00017912  rf_select_band_regs
 * ====================================================================== */

void rf_select_band_regs(uint param_1)

{
  int iVar1;
  undefined4 uVar2;
  
  *(uint *)(DAT_00017cd8 + -0x90) = (param_1 & 3) + DAT_00017ce4;
  uVar2 = DAT_00017ce0;
  iVar1 = DAT_00017cd8;
  if ((param_1 == 0) || ((param_1 != 2 && (param_1 != 3)))) {
    *(int *)(DAT_00017cd8 + -0x8c) = DAT_00017cdc;
  }
  else {
    *(int *)(DAT_00017cd8 + -0x8c) = DAT_00017cdc + -0x118;
  }
  *(undefined4 *)(iVar1 + -0x88) = uVar2;
  return;
}



/* ======================================================================
 * 00017922  rf_cal_path_setup
 * ====================================================================== */

void rf_cal_path_setup(int param_1,int param_2,int param_3)

{
  uint *puVar1;
  int iVar2;
  uint uVar3;
  uint uVar4;
  uint uVar5;
  
  iVar2 = DAT_00017ce8;
  puVar1 = (uint *)(DAT_00017ce8 + 0x50);
  uVar3 = *(uint *)(DAT_00017ce8 + 0x34);
  if (param_1 == 0) {
    *(undefined4 *)(DAT_00017ce8 + 4) = *(undefined4 *)(param_3 + 0x224);
    *(undefined4 *)(iVar2 + 0x50) = *(undefined4 *)(param_3 + 0x228);
    *(undefined4 *)(iVar2 + 0x34) = *(undefined4 *)(param_3 + 0x22c);
    return;
  }
  if (param_1 != 1) {
    return;
  }
  *(uint *)(DAT_00017ce8 + 4) = *(uint *)(DAT_00017ce8 + 4) | 2;
  uVar4 = (((DAT_00017cec | *puVar1 & 0xfffffff1 | 1) & 0xfff97fff | 0x6000) + 0x40000 & 0xffe7ffff)
          + 0x100000;
  uVar5 = uVar4 & 0xffffff | 0x200000;
  if (param_2 == 0) {
    uVar5 = uVar4 & 0x3fffff | 0x210000;
LAB_000179ba:
    uVar5 = uVar5 & 0xfffffdff;
  }
  else {
    if (param_2 == 1) {
      if (*(char *)(DAT_00017cf0 + 2) == '\x01') {
        uVar5 = uVar4 & 0x7effff | 0x600000;
      }
      else {
        uVar5 = uVar4 & 0xbeffff | 0xa00000;
      }
    }
    else {
      if (param_2 == 0) goto LAB_000179ba;
      if (param_2 != 1) goto LAB_000179cc;
    }
    uVar5 = uVar5 | 0x200;
  }
LAB_000179cc:
  *(uint *)(iVar2 + 0x50) = uVar5;
  *(uint *)(DAT_00017ce8 + 0x34) = (uVar3 | DAT_00017cf4 | DAT_00017cf4 + 1) & 0x1fff | 0x1800;
  fw_delay_loop(10);
  if (param_2 == 0) {
    *(uint *)(iVar2 + 0x50) = uVar5 & 0xfffeffff;
  }
  return;
}



/* ======================================================================
 * 00017a14  rf_clear_iq_dac
 * ====================================================================== */

void rf_clear_iq_dac(void)

{
  uint uVar1;
  uint in_r3;
  undefined4 local_8;
  
  local_8 = in_r3 & 0xfffffe00;
  uVar1 = local_8 & ~DAT_00017cf8;
  local_8._3_1_ = (undefined1)(uVar1 >> 0x18);
  local_8._0_3_ = (uint3)(ushort)uVar1;
  *(uint *)(DAT_00017cd8 + -0x6c) = local_8 & ~(DAT_00017cf8 << 0x10);
  return;
}



/* ======================================================================
 * 00017a3e  rf_compute_iq_gain_corr
 * ====================================================================== */

void rf_compute_iq_gain_corr(int param_1)

{
  int iVar1;
  int iVar2;
  undefined4 uVar3;
  uint uVar4;
  uint uVar5;
  uint *puVar6;
  int iVar7;
  int iVar8;
  int iVar9;
  int iVar10;
  uint uVar11;
  int iVar12;
  int iVar13;
  uint uVar14;
  
  uVar14 = 0;
  iVar12 = 0;
  do {
    iVar1 = *(int *)(DAT_00017cfc + iVar12 * 4) * 4;
    puVar6 = (uint *)(iVar1 + DAT_00017d00);
    uVar11 = 0;
    iVar7 = DAT_00017d00 + 0x80;
    iVar13 = iVar12 * 0x10 + param_1;
    iVar2 = *(int *)(iVar13 + 0x14);
    iVar8 = iVar2;
    if (iVar2 < 0) {
      iVar8 = -iVar2;
    }
    iVar9 = *(int *)(iVar13 + 0x18);
    iVar10 = iVar9;
    if (iVar9 < 0) {
      iVar10 = -iVar9;
    }
    if (iVar10 < iVar8) {
      iVar8 = iVar2;
      if (iVar2 < 0) {
        iVar8 = -iVar2;
      }
    }
    else {
      iVar8 = iVar9;
      if (iVar9 < 0) {
        iVar8 = -iVar9;
      }
    }
    for (; iVar8 < 0x40000; iVar8 = iVar8 << 1) {
      uVar11 = uVar11 + 1;
    }
    iVar2 = iVar2 << (uVar11 & 0xff);
    *(int *)(iVar13 + 0x14) = iVar2;
    *(int *)(iVar13 + 0x18) = iVar9 << (uVar11 & 0xff);
    uVar3 = fw_div_scaled(0xe0000000,iVar2);
    uVar4 = fw_sat_round_shift(uVar3,10,0xc);
    uVar3 = fw_div_scaled(0xe0000000,*(undefined4 *)(iVar13 + 0x18));
    uVar5 = fw_sat_round_shift(uVar3,10,0xc);
    uVar14 = (uVar14 & 0xfffffe00 | uVar4 & 0x1ff) & DAT_00017d04 | (uVar5 & 0x1ff) << 0x10;
    *puVar6 = uVar14;
    uVar4 = *(uint *)(DAT_00017cf0 + 0x2c) >> 8;
    uVar5 = (uVar4 | *(uint *)(DAT_00017cf0 + 0x2c) << 0x18) + uVar11 * -0x10000000 >> 0x18;
    uVar4 = (uVar5 | uVar4 * 0x100) >> 4;
    *(uint *)(iVar1 + iVar7) = (uVar4 | uVar5 << 0x1c) + uVar11 * -0x10000000 >> 0x1c | uVar4 * 0x10
    ;
    iVar12 = iVar12 + 1;
  } while (iVar12 < 0xc);
  return;
}



/* ======================================================================
 * 00017b1c  rf_write_iq_corr_regs
 * ====================================================================== */

void rf_write_iq_corr_regs(int param_1,int param_2,int param_3)

{
  undefined1 uVar1;
  undefined1 uVar2;
  undefined4 *puVar3;
  undefined4 *puVar4;
  int iVar5;
  uint uVar6;
  int iVar7;
  int iVar8;
  undefined4 local_2c;
  undefined4 local_28;
  undefined4 local_24;
  
  iVar7 = DAT_00017cd8 + -0x68;
  puVar3 = (undefined4 *)(DAT_00017cd8 + 0x18);
  puVar4 = (undefined4 *)(DAT_00017cd8 + 0x1c);
  if (param_2 == 1) {
    uVar6 = 0;
    iVar5 = *DAT_00017cfc * 4;
    do {
      iVar8 = uVar6 * 0x10 + param_1;
      uVar1 = fw_saturate_signed(*(undefined4 *)(iVar8 + 0x114),8);
      uVar2 = fw_saturate_signed(*(undefined4 *)(iVar8 + 0x118),8);
      local_24._0_2_ = CONCAT11(uVar2,uVar1);
      *(undefined4 *)(iVar5 + iVar7) = local_24;
      uVar6 = uVar6 + 1;
      iVar5 = DAT_00017cfc[uVar6] << 2;
    } while (uVar6 < 0xc);
  }
  if (param_3 == 1) {
    uVar1 = fw_saturate_signed(*(undefined4 *)(param_1 + 0x21c),8);
    uVar2 = fw_saturate_signed(*(undefined4 *)(param_1 + 0x220),8);
    local_28._0_2_ = CONCAT11(uVar2,uVar1);
    uVar1 = fw_saturate_signed(*(undefined4 *)(param_1 + 0x214),8);
    uVar2 = fw_saturate_signed(*(undefined4 *)(param_1 + 0x218),8);
    local_2c._0_2_ = CONCAT11(uVar2,uVar1);
    *puVar3 = local_28;
    *puVar4 = local_2c;
  }
  return;
}



/* ======================================================================
 * 00017bc4  rf_measure_iq
 * ====================================================================== */

void rf_measure_iq(uint param_1,uint param_2,undefined4 *param_3,uint param_4)

{
  int *piVar1;
  undefined4 uVar2;
  int *piVar3;
  int iVar4;
  int *piVar5;
  undefined4 local_10;
  
  local_10._2_2_ =
       (ushort)(1 << (param_1 & 0xff)) & 0xfff | (ushort)((param_4 & DAT_00017d08) >> 0x10);
  local_10._0_2_ = (ushort)(byte)(param_4 & DAT_00017d08);
  local_10 = (local_10 & 0xffffff08 | 8 | (param_2 & 1) << 2) + 1;
  piVar1 = (int *)(DAT_00017cd8 + -0x90);
  piVar3 = (int *)(DAT_00017cd8 + -0x74);
  piVar5 = (int *)(DAT_00017cd8 + -0x70);
  *piVar1 = local_10;
  while (-1 < (int)(local_10 << 0x1b)) {
    local_10 = *piVar1;
  }
  iVar4 = *piVar5;
  uVar2 = fw_sat_round_shift((*piVar3 << 9) >> 9,0xc,0x17,local_10 << 0x1b,iVar4,local_10);
  *param_3 = uVar2;
  uVar2 = fw_sat_round_shift((iVar4 << 9) >> 9,0xc,0x17);
  param_3[1] = uVar2;
  return;
}



/* ======================================================================
 * 00017c46  rf_set_iq_dac
 * ====================================================================== */

void rf_set_iq_dac(undefined1 param_1,undefined1 param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  
  uVar1 = CONCAT31((int3)((uint)param_4 >> 8),param_1) & 0xfffffdff;
  *(uint *)(DAT_00017cd8 + -0x6c) =
       CONCAT13((char)(uVar1 >> 0x18),CONCAT12(param_2,(short)uVar1)) & 0xfdffffff | 0x1000100;
  return;
}



/* ======================================================================
 * 00017c74  rf_calibrate_iq_dc
 * ====================================================================== */

void rf_calibrate_iq_dc(int param_1,int param_2)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  undefined4 uVar4;
  undefined4 uVar5;
  undefined4 uVar6;
  undefined4 uVar7;
  int iVar8;
  int iVar9;
  int local_4c;
  int local_48;
  int local_44;
  int local_40;
  int local_3c;
  int local_38;
  undefined4 local_34;
  undefined4 local_30;
  uint local_2c;
  int local_28;
  int local_24;
  int *local_20;
  int local_1c;
  int local_18;
  
  iVar1 = DAT_00017d0c;
  local_1c = param_1;
  local_18 = param_2;
  rf_cal_gate(1);
  iVar9 = DAT_00017ce8;
  *(undefined4 *)(iVar1 + 0x224) = *(undefined4 *)(DAT_00017ce8 + 4);
  *(undefined4 *)(iVar1 + 0x228) = *(undefined4 *)(iVar9 + 0x50);
  *(undefined4 *)(iVar1 + 0x22c) = *(undefined4 *)(iVar9 + 0x34);
  *(undefined4 *)(iVar1 + 0x230) = *(undefined4 *)(DAT_00017ce8 + 0x84);
  *(undefined4 *)(iVar1 + 0x234) = *(undefined4 *)(iVar9 + 0x6c);
  uVar2 = *(uint *)(DAT_00017cd8 + -0x90);
  if (*(char *)(DAT_00017cf0 + 0x10) == '\0') {
    *(undefined4 *)(DAT_00017cf0 + 0x2c) = *(undefined4 *)(DAT_00017d00 + 0x80);
  }
  rf_select_band_regs(0);
  local_20 = &local_44;
  if (local_18 == 1) {
    rf_cal_path_setup(1,0,iVar1);
    local_24 = 0x44;
    local_28 = 0x44;
    local_30 = 1;
    local_2c = 0xe;
    iVar9 = 0;
    local_34 = 1;
    do {
      rf_set_test_tone((int)(char)*(undefined4 *)(DAT_00017f1c + (iVar9 / 2) * 4));
      rf_set_iq_dac(0x11,0x11);
      rf_measure_iq(0xb,1,&local_4c);
      rf_set_iq_dac(local_30,local_34);
      rf_measure_iq(0xb,1,local_20);
      iVar3 = (local_44 - local_4c) * -0x100;
      iVar8 = iVar9 * 8 + iVar1;
      *(int *)(iVar8 + 0x14) = iVar3;
      local_38 = (local_40 - local_48) * -0x100;
      *(int *)(iVar8 + 0x18) = local_38;
      if (iVar3 == 0) {
        *(undefined4 *)(iVar8 + 0x14) = 1;
      }
      iVar3 = fw_div_scaled(-local_4c << (local_2c & 0xff),*(undefined4 *)(iVar8 + 0x14));
      local_3c = iVar8 + 0x100;
      *(int *)(iVar8 + 0x114) = iVar3 + local_24;
      if (local_38 == 0) {
        *(undefined4 *)(iVar8 + 0x18) = 1;
      }
      iVar3 = fw_div_scaled(-local_48 << (local_2c & 0xff),*(undefined4 *)(iVar8 + 0x18));
      iVar9 = iVar9 + 2;
      *(int *)(local_3c + 0x18) = iVar3 + local_28;
    } while (iVar9 < 0x18);
    *(undefined4 *)(iVar1 + 0x214) = 0;
    *(undefined4 *)(iVar1 + 0x218) = 0;
    *(undefined4 *)(iVar1 + 0x21c) = 0;
    *(undefined4 *)(iVar1 + 0x220) = 0;
    rf_cal_save_refs(*(undefined4 *)(iVar1 + 0x1c4),*(undefined4 *)(iVar1 + 0x1c8),
                     *(undefined4 *)(iVar1 + 0xc4),*(undefined4 *)(iVar1 + 200));
    *(undefined1 *)(DAT_00017f20 + 0x10) = 1;
  }
  if ((local_1c == 1) && (*(char *)(DAT_00017f20 + 0x10) == '\x01')) {
    uVar4 = rf_cal_get_ref0();
    uVar5 = rf_cal_get_ref1();
    uVar6 = rf_cal_get_ref2();
    uVar7 = rf_cal_get_ref3();
    uVar4 = fw_sat_round_shift(uVar4,6,8);
    uVar5 = fw_sat_round_shift(uVar5,6,8);
    rf_set_test_tone(0x20);
    rf_set_iq_dac(uVar4,uVar5);
    rf_measure_iq(0xb,1,&local_4c);
    rf_cal_path_setup(1,1,iVar1);
    rf_set_iq_dac(uVar4,uVar5);
    rf_measure_iq(0xb,0,local_20);
    uVar4 = fw_div_scaled((local_4c - local_44) * 0x4000,uVar6);
    *(undefined4 *)(iVar1 + 0x21c) = uVar4;
    uVar4 = fw_div_scaled((local_48 - local_40) * 0x4000,uVar7);
    *(undefined4 *)(iVar1 + 0x220) = uVar4;
  }
  rf_write_iq_corr_regs(iVar1,local_18,local_1c);
  if (local_18 == 1) {
    rf_compute_iq_gain_corr(iVar1);
  }
  rf_clear_iq_dac();
  rf_cal_path_setup(0,0,iVar1);
  rf_select_band_regs(uVar2 & 3);
  rf_set_test_tone(0xffffffff);
  rf_cal_gate(0);
  return;
}



/* ======================================================================
 * 00017ee6  phy_agc_enable
 * ====================================================================== */

void phy_agc_enable(int param_1)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  iVar1 = DAT_00017f24;
  if (param_1 == 1) {
    uVar3 = *(uint *)(DAT_00017f24 + 0x38) | 0x1800;
    *(undefined4 *)(DAT_00017f28 + 0x20) = 0;
    iVar2 = DAT_00017f2c;
    if (*(char *)(DAT_00017f20 + 2) == '\x01') {
      *(undefined4 *)(DAT_00017f2c + 0x2c) = 0;
    }
    else {
      *(undefined4 *)(DAT_00017f2c + 0x2c) = DAT_00017f30;
    }
    *(undefined4 *)(iVar2 + 0x28) = 0;
  }
  else {
    uVar3 = *(uint *)(DAT_00017f24 + 0x38) & 0xffffe7ff;
  }
  *(uint *)(iVar1 + 0x38) = uVar3;
  return;
}



/* ======================================================================
 * 00017f34  rf_op_dispatch2
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00017f58) */
/* WARNING: Removing unreachable block (ram,0x00017f58) */

void rf_op_dispatch2(int param_1)

{
  uint uVar1;
  
  uVar1 = *(uint *)(param_1 + 4);
                    /* WARNING: Could not recover jumptable at 0x00017f58. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  if (DAT_00017f5c <= uVar1) {
    uVar1 = (uint)DAT_00017f5c;
  }
  (*(code *)((uint)*(byte *)(uVar1 + 0x17f5d) * 2 + 0x17f5d))();
  return;
}



/* ======================================================================
 * 000183e0  rf_pll_restart
 * ====================================================================== */

void rf_pll_restart(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  
  iVar1 = DAT_000187c8;
  uVar3 = *(uint *)(DAT_000187c8 + 0x38) & 0xffffdfff;
  *(uint *)(DAT_000187c8 + 0x38) = uVar3;
  fw_udelay_count(1);
  *(uint *)(iVar1 + 0x38) = uVar3 | 0x2000;
  if (*(char *)(DAT_000187cc + 0x14) != '\0') {
    fw_udelay_count(0x78);
  }
  if (*(char *)(DAT_000187d0 + 2) == '\0') {
    *(uint *)(iVar1 + 4) = *(uint *)(iVar1 + 4) & 0xffffffbf;
    iVar2 = DAT_000187c8;
    *(uint *)(DAT_000187c8 + -0x30) = *(uint *)(DAT_000187c8 + -0x30) & 0xfffffdff;
    fw_udelay_count(10);
    *(uint *)(iVar1 + 4) = *(uint *)(iVar1 + 4) | 0x40;
    *(uint *)(iVar2 + -0x30) = *(uint *)(iVar2 + -0x30) | 0x200;
    fw_udelay_count(10);
  }
  return;
}



/* ======================================================================
 * 0001843e  rf_load_band_regs
 * ====================================================================== */

void rf_load_band_regs(int param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  iVar1 = DAT_000187c8;
  *(undefined4 *)(DAT_000187c8 + -0x7c) = *(undefined4 *)(param_1 + 0x30);
  *(undefined4 *)(iVar1 + -0x78) = *(undefined4 *)(param_1 + 0x34);
  *(undefined4 *)(iVar1 + -0x58) = *(undefined4 *)(param_1 + 0x38);
  *(undefined4 *)(iVar1 + -0x50) = *(undefined4 *)(param_1 + 0x3c);
  *(undefined4 *)(DAT_000187c8 + 0x7c) = *(undefined4 *)(param_1 + 0x40);
  iVar2 = DAT_000187c8;
  *(undefined4 *)(DAT_000187c8 + -0x14) = *(undefined4 *)(param_1 + 0x44);
  iVar3 = DAT_000187c8;
  *(undefined4 *)(DAT_000187c8 + 4) = *(undefined4 *)(param_1 + 0x48);
  *(undefined4 *)(iVar1 + -0x4c) = *(undefined4 *)(param_1 + 0x4c);
  *(undefined4 *)(iVar1 + -0x60) = *(undefined4 *)(param_1 + 0x54);
  *(undefined4 *)(iVar2 + -0x30) = *(undefined4 *)(param_1 + 0x50);
  *(undefined4 *)(iVar1 + -0x5c) = *(undefined4 *)(param_1 + 0x58);
  fw_udelay_count(10);
  iVar1 = DAT_000187d4;
  *(undefined4 *)(DAT_000187d4 + 0x24) = *(undefined4 *)(param_1 + 100);
  phy_cal_cmd_stop();
  iVar2 = DAT_000187d8;
  *(undefined4 *)(DAT_000187d8 + 0xc) = *(undefined4 *)(param_1 + 0x68);
  *(undefined4 *)(iVar2 + 0x1c) = *(undefined4 *)(param_1 + 0x7c);
  *(undefined4 *)(iVar1 + 0x28) = *(undefined4 *)(param_1 + 0x80);
  if (*(int *)(param_1 + 0x74) == 1) {
    *(undefined4 *)(DAT_000187d8 + 0xa8) = *(undefined4 *)(param_1 + 0x6c);
  }
  if (*(int *)(param_1 + 0x78) == 1) {
    *(undefined4 *)(DAT_000187d8 + 0x68) = *(undefined4 *)(param_1 + 0x70);
  }
  *(undefined4 *)(iVar2 + 4) = *(undefined4 *)(param_1 + 0x5c);
  *(undefined4 *)(iVar3 + 0x34) = *(undefined4 *)(param_1 + 0x60);
  rf_pll_restart();
  *DAT_000187dc = *(undefined4 *)(param_1 + 0x84);
  fw_udelay_count(6);
  return;
}



/* ======================================================================
 * 000184d4  rf_dft_correlate_samples
 * ====================================================================== */

void rf_dft_correlate_samples(int param_1)

{
  uint uVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  int iVar7;
  int iVar8;
  int iVar9;
  int iVar10;
  int iVar11;
  undefined4 local_38;
  undefined4 local_34;
  undefined4 local_30;
  undefined4 local_2c;
  undefined4 local_28;
  undefined4 local_24;
  undefined4 local_20;
  
  local_20 = 0;
  local_24 = 0;
  local_2c = 0;
  local_38 = 0;
  local_34 = 0;
  iVar11 = 0;
  local_30 = 0;
  local_28 = 0;
  iVar6 = *(int *)(param_1 + 0xd8);
  iVar7 = *(int *)(param_1 + 0xd4);
  iVar8 = *(int *)(param_1 + 0xdc);
  iVar10 = 0;
  iVar5 = ((*(uint *)(param_1 + 0x8c) & 0x7fff) >> 0xd) + 1;
  fw_bit_length(*(uint *)(param_1 + 0x8c) & 0xff);
  uVar1 = fw_bit_length(*(uint *)(param_1 + 0x8c) >> 0x16);
  iVar9 = 0;
  do {
    uVar2 = *(uint *)(iVar9 * 4 + param_1 + 0x18c);
    if ((uint)(DAT_000187e0 << (*(ushort *)(DAT_000187d0 + 0x36) & 0xff)) < uVar2) {
      uVar2 = uVar2 - (0x1000 << (*(ushort *)(DAT_000187d0 + 0x36) & 0xff));
    }
    iVar3 = (int)(short)((int)(uVar2 << 4) >> (uVar1 & 0xff));
    iVar4 = *(int *)(param_1 + 0x94);
    if (iVar4 == 1) {
LAB_00018566:
      iVar4 = rf_scale_by_tbl_a(local_30,iVar3);
      local_20 = iVar4 + local_20;
      iVar4 = rf_scale_by_tbl_b(local_30,iVar3);
      local_2c = local_2c - iVar4;
      local_30 = local_30 + ((iVar5 * iVar7 & 0xffU) >> 1) & 0x3f;
LAB_0001858e:
      iVar4 = rf_scale_by_tbl_a(local_38,iVar3);
      local_28 = iVar4 + local_28;
      iVar4 = rf_scale_by_tbl_b(local_38,iVar3);
      iVar11 = iVar11 - iVar4;
      local_38 = local_38 + ((iVar8 * iVar5 & 0xffU) >> 1) & 0x3f;
      if (*(int *)(param_1 + 0x94) < 9) goto LAB_000185b8;
    }
    else {
      if (7 < iVar4) {
        if (iVar4 < 9) goto LAB_00018566;
        goto LAB_0001858e;
      }
LAB_000185b8:
      iVar4 = rf_scale_by_tbl_a(local_34,iVar3);
      local_24 = iVar4 + local_24;
      iVar3 = rf_scale_by_tbl_b(local_34,iVar3);
      iVar10 = iVar10 - iVar3;
      local_34 = local_34 + ((iVar5 * iVar6 & 0xffU) >> 1) & 0x3f;
    }
    iVar9 = iVar9 + 1;
    if (0x3f < iVar9) {
      *(short *)(param_1 + 0xfc) = (short)((uint)local_20 >> 0x10);
      *(short *)(param_1 + 0xfe) = (short)((uint)local_2c >> 0x10);
      *(short *)(param_1 + 0x100) = (short)((uint)local_24 >> 0x10);
      *(short *)(param_1 + 0x102) = (short)((uint)iVar10 >> 0x10);
      *(short *)(param_1 + 0x104) = (short)((uint)local_28 >> 0x10);
      *(short *)(param_1 + 0x106) = (short)((uint)iVar11 >> 0x10);
      return;
    }
  } while( true );
}



/* ======================================================================
 * 00018610  rf_capture_adc_samples
 * ====================================================================== */

void rf_capture_adc_samples(int param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  
  iVar3 = DAT_000187d4;
  iVar4 = 0;
  do {
    uVar2 = *(uint *)(iVar3 + 0x2c);
    iVar4 = iVar4 + 1;
    fw_udelay_count(1);
    iVar1 = DAT_000187d4;
  } while (((uVar2 & 0xffff) >> 0xf & (uint)(iVar4 < DAT_000187e4)) != 0);
  iVar3 = 0;
  do {
    iVar4 = iVar3 * 4;
    iVar3 = iVar3 + 1;
    *(undefined4 *)(iVar4 + param_1 + 0x18c) = *(undefined4 *)(iVar4 + iVar1 + 0x44);
  } while (iVar3 < 0x40);
  return;
}



/* ======================================================================
 * 00018654  phy_set_reg2c_bit8
 * ====================================================================== */

void phy_set_reg2c_bit8(int param_1)

{
  uint uVar1;
  
  uVar1 = *(uint *)(param_1 + 0x8c) | 0x100;
  *(uint *)(param_1 + 0x8c) = uVar1;
  *(uint *)(DAT_000187d4 + 0x2c) = uVar1;
  return;
}



/* ======================================================================
 * 00018666  phy_program_iq_corr
 * ====================================================================== */

void phy_program_iq_corr(uint *param_1)

{
  uint uVar1;
  
  uVar1 = DAT_000187e8;
  *(uint *)(DAT_000187d8 + 0x68) = *param_1 & 0xfff & DAT_000187e8 | (param_1[1] & 0xfff) << 0x10;
  *(uint *)(DAT_000187d8 + 0xa8) = (param_1[3] & 0xfff) << 0x10 | param_1[2] & 0xfff & uVar1;
  return;
}



/* ======================================================================
 * 0001869e  rf_program_synth_freq
 * ====================================================================== */

void rf_program_synth_freq(int param_1,int param_2)

{
  uint uVar1;
  undefined4 uVar2;
  uint uVar3;
  uint uVar4;
  uint uVar5;
  undefined4 uVar6;
  undefined8 uVar7;
  longlong lVar8;
  longlong lVar9;
  longlong lVar10;
  undefined8 uVar11;
  
  uVar5 = *(uint *)(DAT_000187d0 + 0x20);
  if (*(char *)(DAT_000187d0 + 2) == '\x01') {
    uVar3 = DAT_000187ec * param_1;
    uVar5 = uVar5 * 1000;
    uVar4 = 0;
    uVar1 = 0;
    do {
      uVar5 = uVar5 >> 1;
      if (uVar5 <= uVar3) {
        uVar3 = uVar3 - uVar5;
        uVar1 = uVar1 | 1 << (0x14 - uVar4 & 0xff);
      }
      uVar4 = uVar4 + 1;
    } while (uVar4 < 0x15);
    uVar1 = uVar1 | param_2 << 0x15;
  }
  else {
    uVar7 = mul3(*(int *)(DAT_000187c8 + 0x34) + 10,uVar5);
    uVar1 = s64_div((int)uVar7,(int)((ulonglong)uVar7 >> 0x20),DAT_000187f0,0);
    lVar8 = s64_div(uVar5 << 0x19,uVar5 >> 7,1000,0);
    uVar2 = (undefined4)((ulonglong)lVar8 >> 0x20);
    uVar7 = s64_div(uVar1 << 0x19,uVar1 >> 7,(int)lVar8,uVar2);
    lVar9 = u64_mul_full((int)lVar8,uVar2,(int)uVar7,(int)((ulonglong)uVar7 >> 0x20));
    lVar10 = u64_mul_acc_u32(&DAT_00a00000,0,param_1);
    uVar1 = (uint)(lVar9 + lVar10);
    lVar8 = s64_div((int)(lVar8 << 3),(int)((ulonglong)(lVar8 << 3) >> 0x20),10,0);
    uVar2 = (undefined4)(lVar8 << 3);
    uVar6 = (undefined4)((ulonglong)(lVar8 << 3) >> 0x20);
    uVar3 = (int)((ulonglong)(lVar9 + lVar10) >> 0x20) * 8 | uVar1 >> 0x1d;
    uVar1 = uVar1 * 8;
    uVar7 = s64_div(uVar1,uVar3,uVar2,uVar6);
    uVar11 = u64_mul_full((uint)uVar7,(int)((ulonglong)uVar7 >> 0x20),uVar2,uVar6);
    uVar5 = uVar1 - (uint)uVar11;
    uVar11 = s64_div(uVar5 * 0x10000000,
                     ((uVar3 - (int)((ulonglong)uVar11 >> 0x20)) - (uint)(uVar1 < (uint)uVar11)) *
                     0x10000000 | uVar5 >> 4,uVar2,uVar6);
    uVar11 = u64_mul_acc_u32((int)uVar11,(int)((ulonglong)uVar11 >> 0x20),0x200000);
    uVar1 = s64_div((int)uVar11,((uint)uVar7 & 0x7ff) * 0x20000 + (int)((ulonglong)uVar11 >> 0x20),
                    0x10000000,0);
  }
  *(uint *)(DAT_000187c8 + 0x34) = uVar1;
  rf_pll_restart();
  return;
}



/* ======================================================================
 * 000187f4  rf_save_band_regs
 * ====================================================================== */

void rf_save_band_regs(int param_1)

{
  char cVar1;
  undefined4 *puVar2;
  int iVar3;
  uint uVar4;
  int iVar5;
  int iVar6;
  
  fw_udelay_count(1);
  puVar2 = DAT_00018bf0;
  *(undefined4 *)(param_1 + 0x84) = *DAT_00018bf0;
  *puVar2 = 0x4000;
  iVar3 = DAT_00018bf4;
  *(undefined4 *)(param_1 + 0x30) = *(undefined4 *)(DAT_00018bf4 + 4);
  *(int *)(iVar3 + 4) = DAT_00018bf8;
  iVar5 = DAT_00018bf4;
  cVar1 = *(char *)(DAT_00018bfc + 2);
  *(undefined4 *)(param_1 + 0x34) = *(undefined4 *)(iVar3 + 8);
  *(undefined4 *)(iVar3 + 8) = 0x8000;
  if (cVar1 == '\x01') {
    fw_udelay_count(1);
    *(undefined4 *)(param_1 + 0x38) = *(undefined4 *)(iVar3 + 0x28);
    *(undefined4 *)(iVar3 + 0x28) = DAT_00018c00;
    iVar6 = DAT_00018bf4;
    *(undefined4 *)(param_1 + 0x40) = *(undefined4 *)(DAT_00018bf4 + 0xfc);
    *(undefined4 *)(iVar6 + 0xfc) = 0;
    *(undefined4 *)(param_1 + 0x44) = *(undefined4 *)(iVar5 + 0x6c);
    *(undefined4 *)(iVar5 + 0x6c) = DAT_00018c04;
    iVar6 = DAT_00018bf4;
    *(undefined4 *)(param_1 + 0x48) = *(undefined4 *)(DAT_00018bf4 + 0x84);
    *(undefined4 *)(iVar6 + 0x84) = DAT_00018c08;
    *(undefined4 *)(param_1 + 0x4c) = *(undefined4 *)(iVar3 + 0x34);
    *(undefined4 *)(iVar3 + 0x34) = DAT_00018c0c;
    *(undefined4 *)(param_1 + 0x54) = *(undefined4 *)(iVar3 + 0x20);
    *(undefined4 *)(iVar3 + 0x20) = DAT_00018c10;
    *(undefined4 *)(param_1 + 0x50) = *(undefined4 *)(iVar5 + 0x50);
    *(undefined4 *)(iVar5 + 0x50) = DAT_00018c14;
    *(undefined4 *)(param_1 + 0x58) = *(undefined4 *)(iVar3 + 0x24);
    *(undefined4 *)(iVar3 + 0x24) = DAT_00018c18;
    fw_udelay_count(10);
    *(undefined4 *)(iVar5 + 0x50) = DAT_00018c1c;
    *(undefined4 *)(DAT_00018bf4 + 0x84) = DAT_00018c20;
    *(undefined4 *)(DAT_00018c24 + 0x24) = 0x5a;
    iVar5 = DAT_00018c28;
    *(undefined4 *)(param_1 + 0x7c) = *(undefined4 *)(DAT_00018c28 + 0x1c);
    uVar4 = (uint)*(ushort *)(DAT_00018c2c + (*(uint *)(param_1 + 0x294) & 0xff) * 2);
    iVar6 = uVar4 * 0x400 + 0x100;
  }
  else {
    *(undefined4 *)(param_1 + 0x38) = *(undefined4 *)(iVar3 + 0x28);
    *(undefined4 *)(iVar3 + 0x28) = DAT_00018c30;
    iVar6 = DAT_00018bf4;
    *(undefined4 *)(param_1 + 0x40) = *(undefined4 *)(DAT_00018bf4 + 0xfc);
    *(undefined4 *)(iVar6 + 0xfc) = 0;
    *(undefined4 *)(param_1 + 0x44) = *(undefined4 *)(iVar5 + 0x6c);
    *(undefined4 *)(iVar5 + 0x6c) = DAT_00018c34;
    iVar6 = DAT_00018bf4;
    *(undefined4 *)(param_1 + 0x48) = *(undefined4 *)(DAT_00018bf4 + 0x84);
    *(undefined4 *)(iVar6 + 0x84) = DAT_00018c38;
    *(undefined4 *)(param_1 + 0x4c) = *(undefined4 *)(iVar3 + 0x34);
    *(undefined4 *)(iVar3 + 0x34) = DAT_00018c3c;
    *(undefined4 *)(param_1 + 0x54) = *(undefined4 *)(iVar3 + 0x20);
    *(uint *)(iVar3 + 0x20) =
         ((*(uint *)(DAT_00018c40 + 0x20) & 0x7ffffff) >> 0x1a) * 0x20000000 + DAT_00018c44;
    *(undefined4 *)(param_1 + 0x50) = *(undefined4 *)(iVar5 + 0x50);
    *(undefined4 *)(iVar5 + 0x50) = DAT_00018c14;
    *(undefined4 *)(param_1 + 0x58) = *(undefined4 *)(iVar3 + 0x24);
    *(undefined4 *)(iVar3 + 0x24) = DAT_00018c48;
    fw_udelay_count(10);
    *(undefined4 *)(iVar5 + 0x50) = DAT_00018c1c;
    *(undefined4 *)(DAT_00018bf4 + 0x84) = DAT_00018c4c;
    iVar5 = DAT_00018c24;
    *(undefined4 *)(param_1 + 100) = *(undefined4 *)(DAT_00018c24 + 0x24);
    *(undefined4 *)(iVar5 + 0x24) = 0x56;
    iVar5 = DAT_00018c28;
    *(undefined4 *)(param_1 + 0x7c) = *(undefined4 *)(DAT_00018c28 + 0x1c);
    uVar4 = (uint)*(ushort *)(DAT_00018c2c + (*(uint *)(param_1 + 0x290) & 0xff) * 2);
    iVar6 = uVar4 * 0x400 + 0xc0;
  }
  *(int *)(iVar5 + 0x1c) = iVar6;
  *(undefined4 *)(param_1 + 0x3c) = *(undefined4 *)(iVar3 + 0x30);
  *(uint *)(iVar3 + 0x30) = (DAT_00018bf8 + 0xf9) - uVar4;
  uVar4 = *(uint *)(DAT_00018bf4 + 0xb4);
  *(uint *)(param_1 + 0x60) = uVar4;
  rf_program_synth_freq(*(undefined4 *)(param_1 + 0xd8),uVar4 >> 0x15);
  iVar5 = DAT_00018c28;
  *(undefined4 *)(param_1 + 0x68) = *(undefined4 *)(DAT_00018c28 + 0xc);
  *(undefined4 *)(iVar5 + 0xc) = 1;
  *(undefined4 *)(param_1 + 0x70) = *(undefined4 *)(DAT_00018c28 + 0x68);
  *(undefined4 *)(param_1 + 0x6c) = *(undefined4 *)(DAT_00018c28 + 0xa8);
  iVar3 = DAT_00018c24;
  *(undefined4 *)(param_1 + 0x74) = 0;
  *(undefined4 *)(param_1 + 0x78) = 0;
  *(undefined4 *)(param_1 + 0x80) = *(undefined4 *)(iVar3 + 0x28);
  *(undefined4 *)(iVar3 + 0x28) = 2;
  phy_cal_cmd_start(*(int *)(param_1 + 0x1c) + -1);
  *(undefined4 *)(param_1 + 0x5c) = *(undefined4 *)(iVar5 + 4);
  *(undefined4 *)(iVar5 + 4) = DAT_00018c50;
  fw_udelay_count(6);
  return;
}



/* ======================================================================
 * 0001899c  rf_apply_channel_settings
 * ====================================================================== */

void rf_apply_channel_settings(int param_1,uint param_2)

{
  uint uVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  undefined4 uVar5;
  undefined4 uVar6;
  int local_304;
  char local_2f5;
  char local_2f4;
  char local_2f3;
  char local_2f1;
  char local_2f0;
  char local_2ef;
  char local_2ee;
  uint local_2ec [4];
  int local_2dc;
  int local_2d8;
  int local_2d4;
  int local_2d0;
  int local_2cc;
  int local_2c8;
  undefined1 auStack_2c4 [20];
  int local_2b0;
  int local_2ac;
  undefined4 local_2a8;
  uint local_29c;
  uint local_258;
  uint local_254;
  int local_250;
  undefined4 local_24c;
  uint local_238;
  uint local_234;
  int local_230;
  undefined1 auStack_22c [20];
  uint local_218;
  uint local_214;
  uint local_210;
  uint local_20c;
  int local_204;
  int local_200;
  int local_1fc;
  int local_1f8;
  undefined4 local_1f0;
  undefined4 local_1ec;
  undefined4 local_1e8;
  undefined4 local_1dc;
  undefined4 local_18c;
  undefined4 local_188;
  undefined4 local_184;
  int local_180;
  int local_17c;
  int local_178;
  uint local_38;
  undefined4 local_34;
  undefined4 local_30;
  uint *local_2c;
  undefined1 *local_28;
  undefined1 *local_24;
  undefined4 local_20;
  int local_1c;
  uint local_18;
  
  if (*(char *)(DAT_00018bfc + 2) == '\0') {
    local_1c = param_1;
    local_18 = param_2;
    bzero_fast(auStack_2c4,0x298);
    bzero_fast(local_2ec,0x28);
    iVar4 = DAT_00018bfc;
    local_20 = 0x20;
    iVar2 = *(int *)(DAT_00018c28 + 4);
    if ((*(char *)(DAT_00018bfc + 0x40) == '\x01') || (*(char *)(DAT_00018bfc + 0x40) == '\x02')) {
      local_2f1 = '\a';
      local_2f0 = -7;
      local_2ef = -5;
      local_2ee = '\x01';
    }
    else {
      local_2f1 = -4;
      local_2f0 = -0xb;
      local_2ef = -4;
      local_2ee = '\0';
    }
    local_2f5 = '\x05';
    local_2f4 = -1;
    local_2f3 = -1;
    local_38 = local_18 >> 0x16;
    uVar3 = local_18 & 0xff;
    *(ushort *)(DAT_00018bfc + 0x32) = (ushort)(local_18 >> 0x16);
    *(short *)(iVar4 + 0x34) = (short)uVar3;
    local_34 = 0xc;
    local_2a8 = 4;
    local_1ec = 0xc;
    local_1f0 = 0x10;
    local_1e8 = 8;
    local_30 = 0x20;
    local_1dc = 1;
    local_29c = (uint)(local_1c == 4);
    local_238 = local_18 & 0xffff9fff | 0x3f0000;
    rf_save_band_regs(auStack_2c4);
    local_2b0 = 0x200;
    local_18c = 0x100;
    local_188 = 0x100;
    local_184 = 0x100;
    local_2ac = 0x800;
    iVar4 = 0;
    local_214 = (local_254 & 0xfffffff) >> 0x10;
    local_20c = (local_258 & 0xfffffff) >> 0x10;
    if (0x7ff < (local_254 & 0xfff)) {
      iVar4 = 0x1000;
    }
    local_218 = (local_254 & 0xfff) - iVar4;
    iVar4 = 0;
    if (0x7ff < local_214) {
      iVar4 = 0x1000;
    }
    local_214 = local_214 - iVar4;
    iVar4 = 0;
    if (0x7ff < (local_258 & 0xfff)) {
      iVar4 = 0x1000;
    }
    local_210 = (local_258 & 0xfff) - iVar4;
    iVar4 = 0;
    if (0x7ff < local_20c) {
      iVar4 = 0x1000;
    }
    local_20c = local_20c - iVar4;
    local_28 = auStack_22c;
    local_2c = &local_234;
    local_24 = (undefined1 *)&local_218;
    do {
      if (1 < (int)local_234) {
        iVar4 = local_234 - 1;
        local_218 = fw_div_scaled(local_204,iVar4);
        local_214 = fw_div_scaled(local_200,iVar4);
        local_210 = fw_div_scaled(local_1fc,iVar4);
        local_20c = fw_div_scaled(local_1f8,iVar4);
      }
      fw_memcpy_bytes_ret(local_28,local_24,0x14);
      local_230 = 0;
      do {
        if (local_230 < 7) {
          local_238 = CONCAT31(local_238._1_3_,(char)(local_38 >> 1));
          local_238 = (local_38 >> 1) << 0x16 | local_238 & 0x3fffff;
        }
        else {
          local_238 = CONCAT31(local_238._1_3_,(char)local_38);
          local_238 = local_238 & 0x3fffff | local_38 << 0x16;
        }
        if (local_230 == 0) {
          if (local_234 == 0) {
            local_238._1_3_ = (undefined3)(local_238 >> 8);
            local_238 = CONCAT31(local_238._1_3_,0xff);
            local_238 = local_238 | DAT_00018c54;
          }
LAB_00018bbe:
          phy_program_iq_corr(local_28);
          phy_set_reg2c_bit8(auStack_2c4);
          if ((local_230 != 0) && (local_230 != 7)) {
            if ((local_230 == 1) && (local_234 == 0)) {
              local_238 = CONCAT31(local_238._1_3_,0xff);
              local_238 = local_238 | DAT_00018c54;
            }
            goto LAB_00018c5a;
          }
        }
        else {
          if ((local_230 == 7) ||
             ((rf_capture_adc_samples(auStack_2c4), local_230 != 6 && (local_230 != 0xc))))
          goto LAB_00018bbe;
LAB_00018c5a:
          rf_dft_correlate_samples(auStack_2c4);
        }
        rf_op_dispatch2(local_2c);
        uVar1 = local_234;
        if ((local_230 == 1) && (local_234 == 0)) {
          local_2dc = local_180;
          local_2d8 = local_17c;
          local_2d4 = local_178;
        }
        local_2ec[0] = local_218;
        local_2ec[1] = local_214;
        local_2ec[2] = local_210;
        local_2ec[3] = local_20c;
        iVar4 = 0;
        do {
          if (DAT_00018f2c < (int)local_2ec[iVar4]) {
            local_2ec[iVar4] = local_2ec[iVar4] - 0x400;
          }
          if (DAT_00018f30 < (int)local_2ec[iVar4 + 2]) {
            local_2ec[iVar4 + 2] = local_2ec[iVar4 + 2] - 0x1000;
          }
          iVar4 = iVar4 + 1;
        } while (iVar4 < 2);
        if ((local_2b0 < (int)local_2ec[0]) || ((int)local_2ec[0] < -local_2b0)) {
          local_218 = 0;
        }
        if ((local_2b0 < (int)local_2ec[1]) || ((int)local_2ec[1] < -local_2b0)) {
          local_214 = 0;
        }
        if ((local_2ac < (int)local_2ec[2]) || ((int)local_2ec[2] < -local_2ac)) {
          local_210 = 0;
        }
        if ((local_2ac < (int)local_2ec[3]) || ((int)local_2ec[3] < -local_2ac)) {
          local_20c = 0;
        }
        local_230 = local_230 + 1;
      } while (local_230 < 0xd);
      if (0 < (int)local_234) {
        local_204 = local_204 + local_218;
        local_200 = local_200 + local_214;
        local_1fc = local_1fc + local_210;
        local_1f8 = local_1f8 + local_20c;
      }
      local_234 = local_234 + 1;
    } while (local_234 < uVar3 * (1 - ((iVar2 << 0x1e) >> 0x1f)));
    if (*(char *)(DAT_00018f34 + 2) == '\0') {
      local_304 = -2;
    }
    else {
      local_304 = (int)local_2f1;
      local_2f3 = local_2ee;
      local_2f5 = local_2f0;
      local_2f4 = local_2ef;
    }
    iVar4 = fw_div_scaled(local_204,uVar1);
    local_218 = iVar4 + local_304;
    iVar4 = fw_div_scaled(local_200,uVar1);
    local_214 = iVar4 + local_2f5;
    iVar4 = fw_div_scaled(local_1fc,uVar1);
    local_210 = iVar4 + local_2f4;
    iVar4 = fw_div_scaled(local_1f8,uVar1);
    local_20c = iVar4 + local_2f3;
    phy_program_iq_corr(local_24);
    iVar4 = DAT_00018f38;
    uVar5 = *(undefined4 *)(DAT_00018f38 + 0x28);
    uVar6 = *(undefined4 *)(DAT_00018f38 + 0x68);
    iVar2 = 1;
    do {
      *(undefined4 *)(iVar2 * 4 + iVar4 + 0x28) = uVar5;
      *(undefined4 *)(iVar2 * 4 + iVar4 + 0x68) = uVar6;
      iVar2 = iVar2 + 1;
    } while (iVar2 < 0x10);
    local_230 = 1;
    local_238 = CONCAT31(local_238._1_3_,0xff);
    local_238 = local_238 | DAT_00018f3c;
    phy_set_reg2c_bit8(auStack_2c4);
    rf_capture_adc_samples(auStack_2c4);
    rf_dft_correlate_samples(auStack_2c4);
    rf_op_dispatch2(local_2c);
    iVar4 = DAT_00018f34;
    local_2d0 = local_180;
    local_2cc = local_17c;
    local_2c8 = local_178;
    if ((local_2d8 <= local_17c >> 2) || (local_180 <= local_17c)) {
      local_250 = 1;
      local_210 = local_258 & 0xfff;
      local_20c = local_258 >> 0x10;
    }
    if ((local_2d4 <= local_178 >> 2) || (local_180 <= local_178)) {
      local_24c = 1;
      local_218 = local_254 & 0xfff;
      local_214 = local_254 >> 0x10;
    }
    if (local_180 <= local_17c * 0x400) {
      if (*(char *)(DAT_00018f34 + 2) == '\0') {
        *(undefined1 *)(DAT_00018f34 + 0x78) = 1;
      }
      else {
        *(undefined1 *)(DAT_00018f34 + 0x7a) = 1;
      }
    }
    if (local_180 <= local_178 * 0x400) {
      if (*(char *)(DAT_00018f34 + 2) == '\0') {
        *(undefined1 *)(iVar4 + 0x79) = 1;
      }
      else {
        *(undefined1 *)(iVar4 + 0x7b) = 1;
      }
    }
    iVar4 = DAT_00018f34;
    if (local_250 == 0) {
      if (*(char *)(DAT_00018f34 + 2) == '\0') {
        *(undefined4 *)(DAT_00018f34 + 0x68) = uVar5;
        *(undefined4 *)(iVar4 + 0x6c) = uVar6;
        *(undefined1 *)(iVar4 + 0xd) = 1;
      }
      else {
        *(undefined4 *)(DAT_00018f34 + 0x70) = uVar5;
        *(undefined4 *)(iVar4 + 0x74) = uVar6;
        *(undefined1 *)(iVar4 + 0x15) = 1;
      }
    }
    rf_load_band_regs(auStack_2c4);
  }
  else {
    *(undefined1 *)(DAT_00018f34 + 0x15) = 1;
  }
  return;
}



/* ======================================================================
 * 00018eb0  rf_reload_iq_regs
 * ====================================================================== */

void rf_reload_iq_regs(int param_1)

{
  uint uVar1;
  undefined4 uVar2;
  undefined4 uVar3;
  int iVar4;
  
  if (param_1 == 5) {
    if (*(char *)(DAT_00018f34 + 2) == '\0') {
      uVar2 = *(undefined4 *)(DAT_00018f34 + 0x68);
      uVar3 = *(undefined4 *)(DAT_00018f34 + 0x6c);
    }
    else {
      uVar2 = *(undefined4 *)(DAT_00018f34 + 0x70);
      uVar3 = *(undefined4 *)(DAT_00018f34 + 0x74);
    }
    uVar1 = 0;
    iVar4 = DAT_00018f38 + 0x40;
    do {
      *(undefined4 *)(uVar1 * 4 + iVar4 + -0x18) = uVar2;
      *(undefined4 *)(uVar1 * 4 + iVar4 + 0x28) = uVar3;
      uVar1 = uVar1 + 1;
    } while (uVar1 < 0x10);
    return;
  }
  rf_apply_channel_settings(param_1,0x7ff0110);
  return;
}



/* ======================================================================
 * 00018f44  phy_cfg_pair_helper
 * ====================================================================== */

void phy_cfg_pair_helper(undefined4 param_1,undefined4 param_2,undefined4 param_3,
                        undefined4 *param_4,undefined4 *param_5)

{
  undefined4 uVar1;
  undefined8 uVar2;
  longlong lVar3;
  
  uVar1 = __udivmoddi4(param_1,param_2,param_3,0,param_1,param_2,param_3,param_4);
  *param_4 = uVar1;
  uVar2 = mul3(uVar1,param_3);
  lVar3 = u64_rsub((int)uVar2,(int)((ulonglong)uVar2 >> 0x20),param_1,param_2);
  uVar1 = __udivmoddi4((int)(lVar3 << 0x15),(int)((ulonglong)(lVar3 << 0x15) >> 0x20),param_3,0);
  *param_5 = uVar1;
  return;
}



/* ======================================================================
 * 00018f80  phy_apply_cfg_if_channel_match
 * ====================================================================== */

void phy_apply_cfg_if_channel_match(uint param_1,int param_2,uint param_3,int param_4)

{
  int iVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  undefined4 uVar5;
  undefined8 uVar6;
  uint local_1c;
  int local_18;
  
  iVar2 = DAT_00019014;
  iVar1 = DAT_00019010;
  if ((*(ushort *)(DAT_00019010 + 0x32) == param_1) && (*(char *)(DAT_00019010 + 0x34) == '\0')) {
    local_18 = *(int *)(DAT_00019010 + 0x24);
    local_1c = *(uint *)(DAT_00019010 + 0x28);
  }
  else {
    local_1c = param_3;
    local_18 = param_4;
    iVar3 = fw_div_scaled(param_2 * *(short *)(DAT_00019014 + 0x3c),1000,param_3,0x3c,param_2);
    uVar4 = rf_settle_time_for_op(param_1);
    *(undefined4 *)(iVar2 + 0x28) = uVar4;
    uVar5 = DAT_00019018;
    if (*(char *)(iVar2 + 2) != '\0') {
      uVar5 = 1000;
    }
    uVar6 = mul3(uVar4,uVar5);
    phy_cfg_pair_helper((int)uVar6,(int)((ulonglong)uVar6 >> 0x20),iVar3 + param_2 * 1000,&local_18,
                        &local_1c);
    *(int *)(iVar1 + 0x24) = local_18;
    *(uint *)(iVar1 + 0x28) = local_1c;
    *(short *)(iVar1 + 0x32) = (short)param_1;
  }
  *(uint *)(DAT_0001901c + 0x34) = local_18 << 0x15 | local_1c;
  rf_pll_restart();
  return;
}



/* ======================================================================
 * 00018ffc  phy_write_reg_neg64
 * ====================================================================== */

void phy_write_reg_neg64(int param_1)

{
  undefined4 uVar1;
  
  if (param_1 == 0) {
    uVar1 = 9;
  }
  else {
    uVar1 = 0;
  }
  *(undefined4 *)(DAT_0001901c + -100) = uVar1;
  return;
}



/* ======================================================================
 * 00019020  rf_init_stage_a
 * ====================================================================== */

void rf_init_stage_a(void)

{
  char cVar1;
  int iVar2;
  int iVar3;
  undefined4 uVar4;
  int iVar5;
  undefined4 uVar6;
  undefined4 *puVar7;
  
  iVar2 = DAT_00019370;
  *(undefined4 *)(DAT_00019370 + 0x30) = 0xa0;
  iVar3 = DAT_00019370;
  cVar1 = *(char *)(DAT_00019374 + 2);
  uVar6 = DAT_0001937c;
  if (cVar1 == '\x01') {
    uVar6 = DAT_00019378;
  }
  *(undefined4 *)(DAT_00019370 + -0x98) = uVar6;
  *(undefined4 *)(iVar3 + -0x90) = 0x1c0000;
  if (*(char *)(DAT_00019374 + 0x5f) == '\0') {
    *(undefined4 *)(iVar3 + -0xa4) = 9;
  }
  else {
    *(undefined4 *)(iVar3 + -0xa4) = 0;
  }
  uVar6 = DAT_00019380;
  *(undefined4 *)(iVar2 + 0x14) = DAT_00019380;
  uVar4 = DAT_00019384;
  *(undefined4 *)(iVar2 + 0x18) = DAT_00019384;
  *(int *)(iVar2 + 0x24) = DAT_00019388;
  *(undefined4 *)(iVar2 + 0x1c) = uVar6;
  *(undefined4 *)(iVar2 + 0x20) = uVar4;
  uVar6 = DAT_00019390;
  if (cVar1 == '\x01') {
    uVar6 = DAT_0001938c;
  }
  *(undefined4 *)(iVar3 + -0x94) = uVar6;
  iVar5 = DAT_00019398;
  if (*(short *)(DAT_00019394 + 0x1c) == 0) {
    *(undefined4 *)(DAT_00019398 + 0x1c) = DAT_0001939c;
    *(undefined4 *)(DAT_00019398 + 0xa8) = 0x10c;
  }
  if (*(byte *)(DAT_00019374 + 0x3f) != 0) {
    *(uint *)(iVar5 + 0x1c) =
         *(byte *)(DAT_00019374 + 0x3f) & 0x7f | *(uint *)(iVar5 + 0x1c) & 0xffffff80;
  }
  *(undefined4 *)(iVar3 + -0xac) = DAT_000193a0;
  iVar3 = DAT_00019370;
  puVar7 = (undefined4 *)(DAT_00019370 + -0x40);
  if (cVar1 == '\x01') {
    *(int *)(DAT_00019370 + -0x54) = DAT_00019388 + 0x41;
    uVar6 = DAT_000193a4;
    *puVar7 = DAT_000193a4;
    *(undefined4 *)(iVar3 + -0x50) = uVar6;
    *(undefined4 *)(iVar3 + -0x4c) = uVar6;
    uVar6 = DAT_000193a8;
    *(undefined4 *)(iVar3 + -0x48) = DAT_000193a8;
    *(undefined4 *)(iVar3 + -0x44) = uVar6;
    *(undefined4 *)(iVar2 + 0x38) = DAT_000193ac;
    *(undefined4 *)(iVar3 + -0x3c) = 0x1000;
    *(undefined4 *)(iVar3 + -0x28) = 0x1000;
    *(undefined4 *)(iVar3 + -0x38) = DAT_000193b0;
    *(undefined4 *)(iVar3 + -0x34) = DAT_000193b4;
    uVar6 = DAT_000193b8;
    *(undefined4 *)(iVar3 + -0x30) = DAT_000193b8;
    *(undefined4 *)(iVar3 + -0x2c) = uVar6;
    uVar6 = DAT_000193bc;
  }
  else {
    *(undefined4 *)(DAT_00019370 + -0x54) = 0;
    *puVar7 = DAT_000193c0;
    uVar6 = DAT_000193c4;
    *(undefined4 *)(iVar3 + -0x50) = DAT_000193c4;
    *(undefined4 *)(iVar3 + -0x4c) = uVar6;
    uVar6 = DAT_000193c8;
    *(undefined4 *)(iVar3 + -0x48) = DAT_000193c8;
    *(undefined4 *)(iVar3 + -0x44) = uVar6;
    *(undefined4 *)(iVar2 + 0x38) = uVar6;
    *(undefined4 *)(iVar3 + -0x3c) = 0x1000;
    *(undefined4 *)(iVar3 + -0x28) = DAT_000193cc;
    *(undefined4 *)(iVar3 + -0x38) = DAT_000193d0;
    *(undefined4 *)(iVar3 + -0x34) = DAT_000193d4;
    *(undefined4 *)(iVar3 + -0x30) = DAT_000193d8;
    *(undefined4 *)(iVar3 + -0x2c) = DAT_000193dc;
    uVar6 = DAT_000193e0;
  }
  *(undefined4 *)(iVar2 + 0x34) = uVar6;
  return;
}



/* ======================================================================
 * 00019116  rf_init_stage_b
 * ====================================================================== */

void rf_init_stage_b(void)

{
  char cVar1;
  int *piVar2;
  int *piVar3;
  int *piVar4;
  int iVar5;
  code *pcVar6;
  undefined *puVar7;
  char *pcVar8;
  
  piVar2 = DAT_00019370;
  DAT_00019370[-6] = 0;
  piVar2[-7] = 0;
  piVar2[-8] = DAT_000193e4;
  piVar2[-9] = 0x300000;
  piVar3 = DAT_00019370;
  *DAT_00019370 = DAT_000193e8;
  piVar4 = DAT_00019370;
  pcVar8 = (char *)(DAT_00019374 + 0x40);
  cVar1 = *pcVar8;
  if (*(char *)(DAT_00019374 + 2) == '\x01') {
    iVar5 = DAT_000193f0;
    if (cVar1 != '\x01') {
      iVar5 = DAT_000193ec;
    }
  }
  else {
    iVar5 = DAT_000193f8;
    if (cVar1 != '\x01') {
      iVar5 = DAT_000193f4;
    }
  }
  DAT_00019370[-0x2e] = iVar5;
  piVar4[-0x2f] = DAT_000193fc;
  piVar4[-0x23] = DAT_00019400;
  fw_delay_loop(10);
  piVar2[-9] = DAT_00019404;
  fw_delay_loop(0x87);
  iVar5 = DAT_00019404 + 0x2f;
  piVar2[-9] = iVar5;
  piVar2[-9] = iVar5 * 0x10000;
  if (*pcVar8 == '\x01') {
    pcVar6 = (code *)0x9200;
  }
  else {
    pcVar6 = rx_buf_free;
  }
  piVar4[-0x2e] = (int)pcVar6;
  piVar4[-0x23] = DAT_00019408;
  piVar4[-0x2f] = 0x304;
  piVar4[-0x2f] = DAT_0001940c;
  piVar2 = DAT_00019370;
  DAT_00019370[-0x1c] = DAT_00019410;
  piVar4[-0x23] = 0x4c0;
  fw_delay_loop(10);
  piVar2[-0x1c] = DAT_00019414;
  *piVar3 = DAT_000193e8 + -2;
  iVar5 = DAT_0001941c;
  *(undefined4 *)(DAT_0001941c + 4) = DAT_00019418;
  *piVar3 = DAT_000193e8 + -1;
  fw_delay_loop(0x32);
  piVar4[-0x2f] = 0x304;
  *piVar3 = DAT_000193e8 + -2;
  if (*(char *)(DAT_00019374 + 2) == '\x01') {
    puVar7 = &DAT_00550000;
  }
  else {
    puVar7 = &DAT_00950000;
  }
  piVar2[-0x1c] = (int)puVar7;
  piVar4[-0x23] = DAT_00019420;
  *(undefined4 *)(iVar5 + 4) = 0;
  fw_delay_loop(10);
  return;
}



/* ======================================================================
 * 000191fe  phy_apply_cfg_pair
 * ====================================================================== */

void phy_apply_cfg_pair(undefined4 param_1)

{
  phy_apply_cfg_if_channel_match
            (param_1,*(undefined4 *)(DAT_00019374 + 0x20),*(undefined4 *)(DAT_00019374 + 0x24));
  return;
}



/* ======================================================================
 * 0001920c  rf_init_stage_c
 * ====================================================================== */

void rf_init_stage_c(void)

{
  int iVar1;
  uint uVar2;
  undefined4 uVar3;
  int iVar4;
  uint uVar5;
  uint uVar6;
  
  iVar1 = DAT_00019370;
  uVar2 = *(uint *)(DAT_00019374 + 0x20);
  uVar6 = 0;
  uVar5 = DAT_00019424 << 1;
  if ((DAT_00019424 <= uVar2) && (uVar6 = 1, uVar5 <= uVar2)) {
    uVar6 = 2;
  }
  if (*(char *)(DAT_00019374 + 2) == '\x01') {
    iVar4 = 7;
    if ((DAT_00019428 <= uVar2) && (iVar4 = 6, uVar5 <= uVar2)) {
      iVar4 = 5;
    }
    iVar4 = iVar4 * 0x4000;
    *(uint *)(DAT_00019370 + -0x14) = iVar4 + 0x1000U | DAT_0001942c;
    *(uint *)(iVar1 + -0x10) = uVar6 + 0x200;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(undefined4 *)(iVar1 + -8) = 0x100000;
    *(uint *)(iVar1 + -0x14) = iVar4 + 0x80U | DAT_0001942c + 0x38;
    *(uint *)(iVar1 + -0x10) = uVar6 + 0x200;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(undefined4 *)(iVar1 + -8) = 0x300000;
    fw_delay_loop(10);
    *(uint *)(iVar1 + -0x14) = iVar4 + 0x40U | DAT_0001942c + 0x7b;
    *(uint *)(iVar1 + -0x10) = uVar6 + 0x8200;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(int *)(iVar1 + -8) = DAT_00019404 + 0x28;
    fw_delay_loop(5);
    *(uint *)(iVar1 + -0x14) = iVar4 + 0x1000000U | DAT_00019430;
    *(uint *)(iVar1 + -0x10) = DAT_000193f8 + 0x3aU | uVar6;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(int *)(iVar1 + -8) = DAT_00019404 + 0x28;
    fw_delay_loop(1);
    *(uint *)(iVar1 + -0x10) = DAT_00019434 | uVar6;
    fw_delay_loop(1);
    *(undefined4 *)(iVar1 + -4) = 0x40000;
    uVar3 = DAT_00019438;
  }
  else {
    iVar4 = 5;
    if (((DAT_0001943c <= uVar2) && (iVar4 = 4, DAT_00019428 <= uVar2)) &&
       (iVar4 = 3, uVar5 <= uVar2)) {
      iVar4 = 2;
    }
    uVar2 = iVar4 * 0x4000;
    *(uint *)(DAT_00019370 + -0x14) = DAT_00019440 | uVar2;
    *(uint *)(iVar1 + -0x10) = uVar6 + 0x200;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(undefined4 *)(iVar1 + -8) = 0;
    *(uint *)(iVar1 + -0x14) = DAT_00019440 + 0x38 | uVar2;
    *(uint *)(iVar1 + -0x10) = uVar6 + 0x200;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(undefined4 *)(iVar1 + -8) = 0x200000;
    fw_delay_loop(10);
    *(uint *)(iVar1 + -0x14) = DAT_00019440 + 0x7b | uVar2;
    *(uint *)(iVar1 + -0x10) = uVar6 + 0x7200;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(undefined4 *)(iVar1 + -8) = DAT_00019444;
    fw_delay_loop(5);
    *(uint *)(iVar1 + -0x14) = uVar2 + 0x1000000 | DAT_00019448;
    *(uint *)(iVar1 + -0x10) = DAT_0001944c | uVar6;
    *(undefined4 *)(iVar1 + -4) = 0;
    *(undefined4 *)(iVar1 + -8) = DAT_00019444;
    fw_delay_loop(1);
    *(uint *)(iVar1 + -0x10) = DAT_00019450 | uVar6;
    fw_delay_loop(1);
    *(undefined4 *)(iVar1 + -4) = 0x40000;
    uVar3 = DAT_00019454;
  }
  *(undefined4 *)(iVar1 + -8) = uVar3;
  fw_delay_loop(0x78);
  return;
}



/* ======================================================================
 * 00019458  rf_init_stage_d
 * ====================================================================== */

void rf_init_stage_d(void)

{
  char cVar1;
  char cVar2;
  int iVar3;
  undefined4 *puVar4;
  undefined4 uVar5;
  undefined4 uVar6;
  int iVar7;
  code *pcVar8;
  
  uVar5 = DAT_00019524;
  puVar4 = DAT_00019520;
  iVar3 = DAT_0001951c;
  cVar1 = *(char *)(DAT_00019518 + 2);
  cVar2 = *(char *)(DAT_00019518 + 0x40);
  DAT_00019520[-0xf] = 0x304;
  if (cVar1 == '\x01') {
    if (cVar2 == '\x01') {
      pcVar8 = (code *)0x9200;
    }
    else {
      pcVar8 = rx_buf_free;
    }
    puVar4[-0xe] = pcVar8;
    puVar4[-8] = DAT_00019528;
    uVar6 = DAT_00019530;
    if (cVar2 != '\x01') {
      uVar6 = DAT_0001952c;
    }
    puVar4[-7] = uVar6;
    puVar4[-10] = 0;
    puVar4[-0xd] = uVar5;
    puVar4[4] = &DAT_00550000;
    puVar4[5] = DAT_00019534;
    puVar4[6] = DAT_00019538;
    uVar5 = DAT_0001953c;
    puVar4[7] = DAT_0001953c;
    puVar4[8] = uVar5;
    puVar4[9] = uVar5;
    puVar4[10] = uVar5;
    iVar7 = DAT_0001951c + -0x50;
  }
  else {
    if (cVar2 == '\x01') {
      pcVar8 = (code *)0x9200;
    }
    else {
      pcVar8 = rx_buf_free;
    }
    puVar4[-0xe] = pcVar8;
    puVar4[-8] = ((*(uint *)(DAT_00019540 + 0x20) & 0x7ffffff) >> 0x1a) * 0x20000000 + DAT_00019544;
    uVar6 = DAT_0001954c;
    if (cVar2 != '\x01') {
      uVar6 = DAT_00019548;
    }
    puVar4[-7] = uVar6;
    puVar4[-10] = 0;
    puVar4[-0xd] = uVar5;
    puVar4[4] = &DAT_00950000;
    puVar4[5] = DAT_00019550;
    puVar4[6] = DAT_00019554;
    uVar5 = DAT_00019558;
    puVar4[7] = DAT_00019558;
    puVar4[8] = uVar5;
    puVar4[9] = uVar5;
    puVar4[10] = uVar5;
    iVar7 = DAT_0001955c;
  }
  puVar4[-3] = iVar7;
  puVar4[-2] = iVar3;
  puVar4[-1] = iVar3;
  uVar5 = DAT_00019560;
  *puVar4 = DAT_00019560;
  puVar4[1] = uVar5;
  puVar4[2] = uVar5;
  puVar4[3] = uVar5;
  return;
}



/* ======================================================================
 * 0001956c  fw_saturate_signed
 * ====================================================================== */

int fw_saturate_signed(int param_1,int param_2)

{
  int iVar1;
  int iVar2;
  
  iVar2 = 1 << (param_2 - 1U & 0xff);
  iVar1 = -iVar2;
  iVar2 = iVar2 + -1;
  if ((param_1 <= iVar2) && (iVar2 = param_1, param_1 < iVar1)) {
    return iVar1;
  }
  return iVar2;
}



/* ======================================================================
 * 00019588  fw_sat_round_shift
 * ====================================================================== */

int fw_sat_round_shift(int param_1,int param_2,int param_3)

{
  int iVar1;
  int iVar2;
  
  if (0 < param_3 - param_2) {
    param_1 = (param_1 >> ((param_3 - param_2) - 1U & 0xff)) + 1 >> 1;
  }
  iVar2 = 1 << (param_2 - 1U & 0xff);
  iVar1 = -iVar2;
  iVar2 = iVar2 + -1;
  if ((param_1 <= iVar2) && (iVar2 = param_1, param_1 < iVar1)) {
    return iVar1;
  }
  return iVar2;
}



/* ======================================================================
 * 000195e0  rf_scale_by_tbl_a
 * ====================================================================== */

int rf_scale_by_tbl_a(int param_1,int param_2)

{
  return param_2 * *(short *)(DAT_00019614 + param_1 * 2) >> 5;
}



/* ======================================================================
 * 000195ec  rf_scale_by_tbl_b
 * ====================================================================== */

int rf_scale_by_tbl_b(int param_1,int param_2)

{
  return param_2 * *(short *)(DAT_00019614 + 0x80 + param_1 * 2) >> 5;
}



/* ======================================================================
 * 000195fa  rf_scale_delta
 * ====================================================================== */

int rf_scale_delta(undefined4 param_1,undefined4 param_2)

{
  int iVar1;
  int iVar2;
  
  iVar1 = fw_bit_length();
  iVar2 = fw_bit_length(param_2);
  return DAT_00019618 * (iVar1 - iVar2) >> 8;
}



/* ======================================================================
 * 0001961c  phy_gain_index_from_value
 * ====================================================================== */

int phy_gain_index_from_value(uint param_1)

{
  int iVar1;
  
  iVar1 = 0x10000;
  if (-1 < (int)(param_1 + DAT_00019a14)) {
    iVar1 = 0x1000000;
    param_1 = param_1 + DAT_00019a14;
  }
  if (-1 < (int)(param_1 + DAT_00019a18)) {
    iVar1 = iVar1 << 4;
    param_1 = param_1 + DAT_00019a18;
  }
  if (-1 < (int)(param_1 + DAT_00019a1c)) {
    iVar1 = iVar1 << 2;
    param_1 = param_1 + DAT_00019a1c;
  }
  if (-1 < (int)(param_1 + DAT_00019a20)) {
    iVar1 = iVar1 << 1;
    param_1 = param_1 + DAT_00019a20;
  }
  if (-1 < (int)(param_1 + DAT_00019a24)) {
    iVar1 = (iVar1 >> 1) + iVar1;
    param_1 = param_1 + DAT_00019a24;
  }
  if (-1 < (int)(param_1 + DAT_00019a28)) {
    iVar1 = (iVar1 >> 2) + iVar1;
    param_1 = param_1 + DAT_00019a28;
  }
  if (-1 < (int)(param_1 + DAT_00019a2c)) {
    iVar1 = (iVar1 >> 3) + iVar1;
    param_1 = param_1 + DAT_00019a2c;
  }
  if (-1 < (int)(param_1 + DAT_00019a30)) {
    iVar1 = (iVar1 >> 4) + iVar1;
    param_1 = param_1 + DAT_00019a30;
  }
  if (-1 < (int)(param_1 + DAT_00019a34)) {
    iVar1 = (iVar1 >> 5) + iVar1;
    param_1 = param_1 + DAT_00019a34;
  }
  if (-1 < (int)(param_1 - 0x3f8)) {
    iVar1 = (iVar1 >> 6) + iVar1;
    param_1 = param_1 - 0x3f8;
  }
  if (-1 < (int)(param_1 - 0x1fe)) {
    iVar1 = (iVar1 >> 7) + iVar1;
    param_1 = param_1 - 0x1fe;
  }
  if ((int)(param_1 << 0x17) < 0) {
    iVar1 = (iVar1 >> 8) + iVar1;
  }
  if ((int)(param_1 << 0x18) < 0) {
    iVar1 = (iVar1 >> 9) + iVar1;
  }
  if ((int)(param_1 << 0x19) < 0) {
    iVar1 = (iVar1 >> 10) + iVar1;
  }
  if ((int)(param_1 << 0x1a) < 0) {
    iVar1 = (iVar1 >> 0xb) + iVar1;
  }
  if ((int)(param_1 << 0x1b) < 0) {
    iVar1 = (iVar1 >> 0xc) + iVar1;
  }
  if ((int)(param_1 << 0x1c) < 0) {
    iVar1 = (iVar1 >> 0xd) + iVar1;
  }
  if ((int)(param_1 << 0x1d) < 0) {
    iVar1 = (iVar1 >> 0xe) + iVar1;
  }
  if ((int)(param_1 << 0x1e) < 0) {
    iVar1 = (iVar1 >> 0xf) + iVar1;
  }
  if ((param_1 & 1) != 0) {
    iVar1 = (iVar1 >> 0x10) + iVar1;
  }
  return iVar1;
}



/* ======================================================================
 * 000196ec  phy_compute_rssi
 * ====================================================================== */

longlong phy_compute_rssi(undefined4 param_1,uint param_2,undefined4 param_3,undefined4 param_4,
                         ushort *param_5,undefined2 *param_6,int param_7,int param_8)

{
  ushort uVar1;
  undefined2 uVar2;
  int iVar3;
  int iVar4;
  int iVar5;
  ushort uVar6;
  undefined2 uVar7;
  uint uVar8;
  int iVar9;
  uint uVar10;
  int iVar11;
  uint uVar12;
  
  iVar9 = (int)*(short *)(*(int *)(DAT_00019a38 + 4) + param_8 * 2 + 0x54);
  if (iVar9 == 0) {
    iVar9 = 0x200;
  }
  if (*(short *)(DAT_00019a3c + 0x1c) == 0) {
    uVar8 = 0xa80;
  }
  else {
    uVar8 = *(uint *)(DAT_00019a3c + 0x2c);
    iVar4 = (uVar8 & 0x7ff) * -0x1000;
    if (-1 < (int)(uVar8 << 0x14)) {
      iVar4 = (uVar8 & 0x7ff) * 0x1000;
    }
    uVar10 = (*(uint *)(DAT_00019a3c + 0x30) & 0xfff) << 4 | uVar8 >> 0x1c;
    uVar8 = (uVar8 & 0xfffffff) >> 0xc;
    if (uVar10 - 0xf00 < DAT_00019a44) goto LAB_00019740;
  }
  uVar10 = 0x2000;
  iVar4 = DAT_00019a40;
LAB_00019740:
  uVar12 = param_2;
  iVar3 = phy_gain_index_from_value((int)(DAT_00019a48 * uVar8) >> 8);
  iVar11 = *(int *)(DAT_00019a38 + 4);
  iVar4 = fw_div_scaled((int)((int)*(short *)(iVar11 + 0x4a) *
                             (DAT_00019a50 * (DAT_00019a4c - uVar12) + iVar4 * -0x10 +
                             uVar10 * -0x1000)) >> 4,1000);
  iVar5 = fw_div_scaled((iVar3 + iVar4) * (int)*(short *)(iVar11 + 0x4e),100);
  iVar4 = fw_div_scaled((iVar3 + iVar4) *
                        (int)*(short *)(iVar11 + 0x4c) * *(int *)(DAT_00019a38 + 0x20),DAT_00019a54)
  ;
  iVar3 = phy_gain_index_from_value(DAT_00019a48 * param_7 >> 4);
  iVar4 = fw_div_scaled(iVar3 << 8,iVar5 + iVar4);
  iVar4 = fw_div_scaled(iVar4 * 1000 + *(short *)(*(int *)(DAT_00019a38 + 4) + 0x52) * 0x100,
                        (int)*(short *)(*(int *)(DAT_00019a38 + 4) + 0x50));
  uVar10 = fw_div_scaled(iVar4 * 100,iVar9);
  uVar8 = 0;
  if (0 < (int)uVar10) {
    uVar8 = uVar10;
  }
  uVar1 = (ushort)(uVar8 >> 8) & (ushort)DAT_00019a58;
  *param_5 = uVar1;
  if ((uVar8 & 0x3ff) != 0) {
    *param_5 = uVar1 + 4;
  }
  uVar1 = 4;
  if (3 < *param_5) {
    uVar1 = *param_5;
  }
  *param_5 = uVar1;
  uVar6 = 0x20;
  if (uVar1 < 0x20) {
    uVar6 = uVar1;
  }
  *param_5 = uVar6;
  uVar8 = fw_div_scaled(uVar8 * iVar9);
  uVar7 = (undefined2)((uVar8 & 0xffffff) >> 8);
  *param_6 = uVar7;
  uVar2 = 100;
  if (99 < (uVar8 & 0xffffff) >> 8) {
    uVar2 = uVar7;
  }
  *param_6 = uVar2;
  return (ulonglong)param_2 << 0x20;
}



/* ======================================================================
 * 00019836  phy_lookup_gain_pair
 * ====================================================================== */

int phy_lookup_gain_pair(int param_1,int param_2,int param_3,undefined4 param_4)

{
  short sVar1;
  undefined4 uVar2;
  int iVar3;
  uint uVar4;
  int iVar5;
  uint uVar6;
  ushort *puVar7;
  
  uVar2 = DAT_00019a54;
  if (*(char *)(DAT_00019a5c + 2) == '\0') {
    puVar7 = (ushort *)(DAT_00019a60 + 0xfc);
    iVar3 = DAT_00019a60;
  }
  else {
    puVar7 = (ushort *)(DAT_00019a60 + 0x100);
    iVar3 = DAT_00019a60 + 0x92;
  }
  sVar1 = *(short *)(iVar3 + param_1 * 2);
  iVar3 = fw_div_scaled((int)((uint)*puVar7 * ((param_2 * 0x580 >> 0x10) + DAT_00019a64)) >> 4,
                        DAT_00019a54,DAT_00019a64,param_4,param_4);
  if (*(short *)(DAT_00019a3c + 0x1c) == 0) {
    uVar6 = 0xffffffb0;
  }
  else {
    uVar4 = *(uint *)(DAT_00019a3c + 0x2c) & 0x7ff;
    uVar6 = -uVar4;
    if (-1 < (int)(*(uint *)(DAT_00019a3c + 0x2c) << 0x14)) {
      uVar6 = uVar4;
    }
  }
  iVar5 = fw_div_scaled((uint)puVar7[1] *
                        (0x1e0 - ((DAT_00019a50 * (DAT_00019a4c - param_3) >> 0x10) - uVar6)),uVar2,
                        0x1e0,DAT_00019a50,param_4);
  return iVar5 + iVar3 + sVar1 + -0x30;
}



/* ======================================================================
 * 000198ae  phy_compute_tx_gain_and_rssi
 * ====================================================================== */

void phy_compute_tx_gain_and_rssi
               (int param_1,int param_2,undefined2 *param_3,undefined4 param_4,undefined4 param_5,
               undefined4 param_6,uint param_7,int *param_8,int *param_9,undefined4 *param_10,
               undefined2 *param_11)

{
  int iVar1;
  ushort local_2c [2];
  undefined1 auStack_28 [4];
  int iStack_24;
  int iStack_20;
  undefined2 *local_1c;
  undefined4 local_18;
  
  iVar1 = DAT_00019a5c;
  *param_9 = *(int *)(DAT_00019a5c + 0x4c);
  *param_8 = *(int *)(iVar1 + 0x48);
  if ((*param_9 == 0) || (*param_9 < 0x23)) {
    *param_9 = *(int *)(iVar1 + 0x4c);
  }
  if ((*param_8 == 0) || (*param_8 < 0x23)) {
    *param_8 = *(int *)(iVar1 + 0x48);
  }
  iStack_24 = param_1;
  iStack_20 = param_2;
  local_1c = param_3;
  local_18 = param_4;
  iVar1 = phy_lookup_gain_pair(param_7,*param_8,*param_9);
  if (param_2 < iVar1) {
    iVar1 = param_2;
  }
  if (iVar1 <= param_1) {
    param_1 = iVar1;
  }
  *param_11 = (short)param_1;
  if (param_7 < 2) {
    iVar1 = 0x40;
  }
  else {
    iVar1 = 0x30;
  }
  phy_compute_rssi(*param_8,*param_9,*param_10,auStack_28,local_2c,local_18,param_1 + iVar1,param_7)
  ;
  *local_1c = *(undefined2 *)(DAT_00019a68 + (uint)(local_2c[0] >> 2) * 2);
  return;
}



/* ======================================================================
 * 00019946  phy_select_rate_tables
 * ====================================================================== */

void phy_select_rate_tables(void)

{
  int iVar1;
  int iVar2;
  
  iVar1 = DAT_00019a38;
  if (*(char *)(DAT_00019a5c + 2) == '\0') {
    *(int *)(DAT_00019a38 + 4) = DAT_00019a60;
    iVar2 = DAT_00019a68 + -0x20;
    *(int *)(iVar1 + 0x18) = iVar2;
  }
  else {
    *(int *)(DAT_00019a38 + 4) = DAT_00019a60 + 0x92;
    iVar2 = DAT_00019a68 + -0x18;
    *(int *)(iVar1 + 0x18) = iVar2;
  }
  *(int *)(iVar1 + 0x1c) = iVar2 + 0x10;
  *(undefined4 *)(iVar1 + 0x20) = 0xffffffff;
  *(undefined1 *)(DAT_00019a38 + 0x35) = 0;
  return;
}



/* ======================================================================
 * 0001997c  phy_set_freq_offset
 * ====================================================================== */

void phy_set_freq_offset(void)

{
  int iVar1;
  int iVar2;
  
  iVar2 = DAT_00019a70;
  if (*(char *)(DAT_00019a5c + 2) == '\0') {
    iVar2 = DAT_00019a6c;
  }
  iVar1 = __udivsi3(*(undefined4 *)(DAT_00019a5c + 0x28),1000);
  iVar2 = (iVar1 - iVar2) * 0x10000 >> 0x10;
  if (*(int *)(DAT_00019a38 + 0x20) != iVar2) {
    *(int *)(DAT_00019a38 + 0x20) = iVar2;
  }
  return;
}



/* ======================================================================
 * 000199a8  phy_compute_temp_from_adc
 * ====================================================================== */

longlong phy_compute_temp_from_adc(short *param_1,undefined2 *param_2)

{
  int iVar1;
  short sVar2;
  undefined2 uVar3;
  uint uVar4;
  int iVar5;
  int iVar6;
  uint uVar7;
  int iVar8;
  uint uVar9;
  undefined4 uStack_20;
  
  uVar7 = (*(uint *)(DAT_00019a3c + 0x2c) & 0xfffffff) >> 0xc;
  uVar9 = (*(uint *)(DAT_00019a3c + 0x34) & 0xfff) << 4 | *(uint *)(DAT_00019a3c + 0x30) >> 0x1c;
  uVar4 = (*(uint *)(DAT_00019a3c + 0x30) & 0xfffffff) >> 0xc;
  if ((((*(short *)(DAT_00019a3c + 0x1c) != 0) && (uVar7 != 0)) && (uVar4 != 0)) && (uVar9 != 0)) {
    iVar5 = fw_div_scaled(DAT_00019a54 * uVar4,uVar7);
    iVar1 = DAT_00019a54;
    iVar6 = fw_div_scaled(DAT_00019a54 * uVar9,uVar7);
    iVar8 = iVar5 + iVar1 + iVar6;
    sVar2 = fw_div_scaled((iVar5 * -0x1e + iVar6 * 0x2a) * 3 + iVar8 * -0xc,DAT_00019a74);
    *param_1 = sVar2;
    iVar5 = fw_div_scaled(iVar8,3);
    iVar6 = fw_div_scaled(sVar2 * 0xc,3);
    uVar3 = fw_div_scaled((iVar5 - iVar6) * 100,iVar1);
    *param_2 = uVar3;
    return (ulonglong)uStack_20 << 0x20;
  }
  return CONCAT44(uStack_20,0xffffffff);
}



/* ======================================================================
 * 00019aae  fw_normalize_to_exp
 * ====================================================================== */

undefined8 fw_normalize_to_exp(int param_1,int param_2,uint *param_3)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  int iVar4;
  uint uVar5;
  
  iVar4 = -param_1;
  if (-1 < param_1) {
    iVar4 = param_1;
  }
  if (param_2 <= iVar4) {
    uVar2 = *param_3;
    iVar1 = -param_2;
    if ((int)uVar2 < 1) {
      while (((param_2 < param_1 || (param_1 < iVar1)) && ((int)*param_3 < 0))) {
        param_1 = param_1 << 1;
        *param_3 = *param_3 + 1;
      }
    }
    else {
      if (param_2 != 0) {
        iVar3 = iVar1;
        if (-1 < param_2) {
          iVar3 = param_2;
        }
        iVar3 = fw_clz(iVar3);
        iVar4 = fw_clz(iVar4);
        uVar2 = iVar3 - iVar4;
      }
      for (; 1 < uVar2; uVar2 = uVar2 - uVar5) {
        uVar5 = 0x1e;
        if (uVar2 < 0x1e) {
          uVar5 = uVar2;
        }
        param_1 = (1 << (uVar5 - 1 & 0xff)) + param_1 >> (uVar5 & 0xff);
        *param_3 = *param_3 - uVar5;
      }
      while (((param_2 < param_1 || (param_1 < iVar1)) && (0 < (int)*param_3))) {
        param_1 = param_1 + 1 >> 1;
        *param_3 = *param_3 - 1;
      }
    }
  }
  return CONCAT44(param_2,param_1);
}



/* ======================================================================
 * 00019b3a  fw_poly_eval_fixed
 * ====================================================================== */

int fw_poly_eval_fixed(int param_1,int param_2,int param_3,short *param_4,int param_5)

{
  undefined4 uVar1;
  int iVar2;
  undefined4 uVar3;
  int iVar4;
  int iVar5;
  int local_54 [10];
  int local_2c;
  int local_28;
  int local_24;
  int iStack_20;
  int local_1c;
  short *local_18;
  
  iVar4 = 0;
  local_2c = 0;
  local_28 = 0;
  local_24 = param_1;
  iStack_20 = param_2;
  local_1c = param_3;
  local_18 = param_4;
  bzero_fast(local_54,0x28);
  while( true ) {
    if (local_24 <= iVar4) {
      return local_28;
    }
    local_2c = *local_18 - param_5;
    iVar5 = *(int *)(local_1c + iVar4 * 4);
    local_54[iVar4] = iVar5;
    if (iVar4 == 0) {
      local_54[0] = fw_normalize_to_exp(local_54[0],0,&local_2c);
    }
    else if ((0 < iVar4) && (param_2 != 0)) {
      iVar2 = -param_2;
      if (-1 < param_2) {
        iVar2 = param_2;
      }
      uVar1 = fw_div_scaled(DAT_00019bfc,iVar2);
      iVar5 = fw_normalize_to_exp(iVar5,uVar1,&local_2c);
      local_54[iVar4] = iVar5;
      for (iVar5 = 0; iVar5 < iVar4; iVar5 = iVar5 + 1) {
        local_54[iVar4] = param_2 * local_54[iVar4];
        local_2c = local_2c + param_5;
        if (iVar5 == iVar4 + -1) {
          iVar2 = local_54[iVar4];
          uVar3 = 0;
        }
        else {
          iVar2 = local_54[iVar4];
          uVar3 = uVar1;
        }
        iVar2 = fw_normalize_to_exp(iVar2,uVar3,&local_2c);
        local_54[iVar4] = iVar2;
      }
    }
    local_28 = local_54[iVar4] + local_28;
    if (param_2 == 0) break;
    iVar4 = iVar4 + 1;
    local_18 = local_18 + 1;
  }
  return local_28;
}



/* ======================================================================
 * 00019c00  wsm_h_1D_stub_returns_zero
 * ====================================================================== */

undefined4 wsm_h_1D_stub_returns_zero(void)

{
  return 0;
}



/* ======================================================================
 * 00019c04  FUN_00019c04
 * ====================================================================== */

void FUN_00019c04(int param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  iVar1 = DAT_00019e1c;
  uVar2 = 0;
  do {
    iVar3 = uVar2 * 4;
    uVar2 = uVar2 + 1;
    *(undefined4 *)(param_1 + iVar3) = *(undefined4 *)(iVar3 + iVar1 + 0x20);
  } while (uVar2 < 8);
  return;
}



/* ======================================================================
 * 00019c1a  FUN_00019c1a
 * ====================================================================== */

/* WARNING: Function: switch8_r3 replaced with injection: switch8_r3 */
/* WARNING (jumptable): Removing unreachable block (ram,0x00019cd4) */
/* WARNING: Removing unreachable block (ram,0x00019cd4) */

undefined4 FUN_00019c1a(undefined4 *param_1)

{
  uint uVar1;
  undefined4 uVar2;
  uint uVar3;
  int iVar4;
  undefined8 uVar5;
  undefined4 local_108;
  uint local_c0;
  uint local_b4 [3];
  uint uStack_a8;
  uint local_94 [32];
  
  bzero_fast(local_94,0x80);
  iVar4 = DAT_00019e1c;
  local_c0 = 0;
  uVar5 = 0;
  param_1[7] = 0;
  if (*(short *)(iVar4 + 0x1c) == 0) {
    *param_1 = 0xffffff64;
    param_1[1] = 0x9d;
    param_1[2] = 0x3a;
    param_1[3] = 0xffffffc4;
    param_1[4] = 0x59;
    param_1[5] = 0xfffffff5;
    param_1[6] = 5;
    return 1;
  }
  FUN_00019c04(local_b4);
  uVar1 = 9;
  do {
    uVar3 = uVar1 + 1;
    local_94[uVar1] = (1 << (uVar1 & 0xff) & local_b4[0]) >> (uVar1 & 0xff);
    uVar1 = uVar3;
  } while ((int)uVar3 < 0xf);
  iVar4 = 9;
  do {
    local_108 = (undefined4)uVar5;
    uVar5 = s64_add_s32(local_108,(int)((ulonglong)uVar5 >> 0x20),
                        local_94[iVar4] << (local_c0 & 0xff));
    local_c0 = local_c0 + 1;
    iVar4 = iVar4 + 1;
  } while (iVar4 < 0xf);
  uVar1 = 0;
  do {
    uVar3 = uVar1 + 1;
    local_94[uVar1] = (1 << (uVar1 & 0xff) & uStack_a8) >> (uVar1 & 0xff);
    uVar1 = uVar3;
  } while ((int)uVar3 < 0x20);
                    /* WARNING: Could not recover jumptable at 0x00019cd4. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  uVar2 = (*(code *)((uint)bRam00019cd9 * 2 + 0x19cd9))();
  return uVar2;
}



/* ======================================================================
 * 00019e24  phy_lookup_by_threshold
 * ====================================================================== */

int phy_lookup_by_threshold(void)

{
  int iVar1;
  int iVar2;
  
  iVar2 = (uint)*(byte *)(DAT_0001a208 + 2) * 8 + DAT_0001a20c;
  for (iVar1 = 0;
      (iVar1 < (int)(uint)*(byte *)(iVar2 + 1) &&
      (*(ushort *)(*(int *)(iVar2 + 4) + iVar1 * 4) <= *(ushort *)(DAT_0001a208 + 6)));
      iVar1 = iVar1 + 1) {
  }
  if (iVar1 == 0) {
    return (int)*(short *)(iVar2 + 2);
  }
  return (int)*(short *)(*(int *)(iVar2 + 4) + iVar1 * 4 + -2);
}



/* ======================================================================
 * 00019e5c  phy_program_gain_entry
 * ====================================================================== */

void phy_program_gain_entry(int param_1,uint param_2,uint param_3)

{
  uint uVar1;
  uint *puVar2;
  uint uVar3;
  
  *(uint *)(param_1 * 4 + DAT_0001a210) = (param_3 & 0x3ff) << 10 | param_2 & 0x3ff & DAT_0001a214;
  uVar3 = 0x800;
  puVar2 = (uint *)(param_1 * 4 + DAT_0001a218);
  uVar1 = uVar3;
  if ((param_2 & 0x3ff) != 0) {
    uVar1 = __udivsi3(DAT_0001a21c,param_2 & 0x3ff);
  }
  if (uVar1 < 0x801) {
    uVar3 = 0x1000 - uVar1;
  }
  *puVar2 = uVar3;
  return;
}



/* ======================================================================
 * 00019e9c  phy_program_gain_for_channel
 * ====================================================================== */

undefined4
phy_program_gain_for_channel(int param_1,undefined4 param_2,uint param_3,undefined4 param_4)

{
  short sVar1;
  uint uVar2;
  uint uVar3;
  uint uVar4;
  uint local_30;
  uint local_2c;
  uint local_28;
  int iStack_24;
  undefined4 local_20;
  uint uStack_1c;
  undefined4 local_18;
  
  local_2c = 0;
  local_30 = 0;
  uVar2 = (int)(((uint)*(ushort *)(DAT_0001a220 + 0x30) + param_1) * 0x10000) >> 0x10;
  uVar3 = 10;
  if (param_3 < 0xb) {
    uVar3 = param_3;
  }
  uVar4 = (uint)*(short *)(DAT_0001a220 + (uint)(1 < uVar3) * 2);
  if ((int)uVar2 < (int)uVar4) {
    uVar4 = uVar2;
  }
  if (*(char *)(DAT_0001a208 + 2) == '\0') {
    sVar1 = *(short *)(DAT_0001a224 + uVar3 * 2);
  }
  else {
    sVar1 = *(short *)(uVar3 * 2 + DAT_0001a224 + 0x92);
  }
  local_28 = (int)sVar1;
  if ((int)uVar2 < (int)sVar1) {
    local_28 = uVar2;
  }
  iStack_24 = param_1;
  local_20 = param_2;
  uStack_1c = param_3;
  local_18 = param_4;
  __udivsi3(*(undefined4 *)(DAT_0001a208 + 0x28),1000);
  phy_compute_tx_gain_and_rssi(local_28,uVar4,&local_2c,&local_30);
  phy_gain_slot_update_fields(uVar2 & 0xffff,0,0,0);
  phy_program_gain_entry(local_20,local_30 & 0xffff,local_2c & 0xffff);
  return 0;
}



/* ======================================================================
 * 00019f56  phy_program_gain_next_slot
 * ====================================================================== */

void phy_program_gain_next_slot(undefined4 param_1,undefined4 param_2,undefined4 param_3)

{
  int iVar1;
  undefined1 uVar2;
  uint uVar3;
  
  iVar1 = DAT_0001a220;
  uVar3 = (uint)*(byte *)(DAT_0001a220 + 0x35);
  phy_program_gain_for_channel
            ((int)(((uint)*(ushort *)(uVar3 * 0x10 + DAT_0001a228 + 2) -
                   (uint)*(ushort *)(DAT_0001a220 + 0x30)) * 0x10000) >> 0x10,uVar3,uVar3,param_1,
             param_2,param_3);
  uVar2 = 0;
  if ((uVar3 + 1 & 0xff) != 0x10) {
    uVar2 = (undefined1)(uVar3 + 1);
  }
  *(undefined1 *)(iVar1 + 0x35) = uVar2;
  return;
}



/* ======================================================================
 * 00019fe2  rf_measure_temp_and_vbat
 * ====================================================================== */

int rf_measure_temp_and_vbat(int param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  int iVar1;
  int iVar2;
  int iVar3;
  uint uVar4;
  int iVar5;
  byte bVar6;
  int iVar7;
  byte bVar8;
  
  iVar7 = 0;
  rf_save_and_set_test_mode();
  iVar1 = DAT_0001a210;
  *(undefined4 *)(DAT_0001a208 + 0x7c) = *(undefined4 *)(DAT_0001a210 + -0x10);
  if (param_1 == 0) {
    bVar8 = 1;
  }
  else {
    bVar8 = 2;
  }
  bVar6 = 0;
  while( true ) {
    iVar2 = DAT_0001a234;
    iVar5 = DAT_0001a208;
    if (bVar8 <= bVar6) {
      *(undefined4 *)(DAT_0001a234 + -0x7c) = *(undefined4 *)(DAT_0001a208 + 0x50);
      *(undefined4 *)(DAT_0001a234 + 4) = *(undefined4 *)(iVar5 + 0x18);
      iVar3 = DAT_0001a234;
      *(undefined4 *)(DAT_0001a234 + -0x14) = *(undefined4 *)(iVar5 + 0x1c);
      *(undefined4 *)(iVar3 + -0x30) = *(undefined4 *)(iVar5 + 0x58);
      *(undefined4 *)(iVar2 + -0x4c) = *(undefined4 *)(iVar5 + 0x54);
      *(undefined4 *)(iVar1 + -0x10) = *(undefined4 *)(iVar5 + 0x7c);
      return iVar7;
    }
    uVar4 = 0;
    *(undefined4 *)(iVar1 + -0x10) = 0;
    if ((bVar6 == 0) || (bVar6 != 1)) {
      iVar5 = 3;
    }
    else {
      iVar5 = 2;
      uVar4 = *(uint *)(DAT_0001a234 + 4);
      *(uint *)(DAT_0001a234 + 4) = uVar4 | 0x300000;
    }
    *(uint *)(iVar1 + -0x10) = uVar4 | iVar5 << 0xe | 0x2080;
    fw_delay_loop(10);
    if (*(int *)(iVar1 + -0x10) << 0x1a < 0) break;
    fw_delay_loop(10);
    iVar5 = *(int *)(DAT_0001a238 + 0x18) * 0x47;
    iVar5 = __udivsi3((iVar5 - *(short *)(DAT_0001a208 + 0x46)) * 1000,
                      (int)*(short *)(DAT_0001a208 + 0x44),iVar5,1000,param_4);
    if (bVar6 == 0) {
      iVar7 = DAT_0001a24c;
      if ((uint)(iVar5 + DAT_0001a244) <= DAT_0001a248) {
        *(int *)(DAT_0001a208 + 0x4c) = iVar5;
        iVar7 = iVar5;
      }
    }
    else if ((bVar6 == 1) && ((uint)(iVar5 + DAT_0001a23c) <= DAT_0001a240)) {
      *(int *)(DAT_0001a208 + 0x48) = iVar5;
    }
    fw_delay_loop(1);
    bVar6 = bVar6 + 1;
  }
  return 0;
}



/* ======================================================================
 * 0001a0d0  rf_convert_adc_to_temp_a
 * ====================================================================== */

void rf_convert_adc_to_temp_a(void)

{
  int iVar1;
  
  iVar1 = __udivsi3((((*(uint *)(DAT_0001a218 + 0x54) & 0xfffff) * 0x38c + 0x80 >> 8) -
                    (int)*(short *)(DAT_0001a208 + 0x46)) * 1000,
                    (int)*(short *)(DAT_0001a208 + 0x44));
  if (iVar1 != 0) {
    *(int *)(DAT_0001a208 + 0x48) = iVar1;
  }
  return;
}



/* ======================================================================
 * 0001a108  rf_convert_adc_to_temp_b
 * ====================================================================== */

void rf_convert_adc_to_temp_b(void)

{
  int iVar1;
  
  iVar1 = __udivsi3((((*(uint *)(DAT_0001a218 + 0x50) & 0xfffff) * 0x38c + 0x80 >> 8) -
                    (int)*(short *)(DAT_0001a208 + 0x46)) * 1000,
                    (int)*(short *)(DAT_0001a208 + 0x44));
  if (iVar1 != 0) {
    *(int *)(DAT_0001a208 + 0x4c) = iVar1;
  }
  return;
}



/* ======================================================================
 * 0001a140  rf_latch_temp_readings
 * ====================================================================== */

void rf_latch_temp_readings(void)

{
  if (-1 < *(int *)(DAT_0001a210 + -0x10) << 0x1a) {
    *(undefined1 *)((uint)*(byte *)(DAT_0001a208 + 0x30) * 0x10 + DAT_0001a228 + 1) = 1;
    rf_convert_adc_to_temp_b();
    rf_convert_adc_to_temp_a();
  }
  return;
}



/* ======================================================================
 * 0001a166  phy_txpower_from_rate_table
 * ====================================================================== */

int phy_txpower_from_rate_table(uint param_1,int param_2)

{
  byte bVar1;
  uint uVar2;
  int iVar3;
  
  iVar3 = *(int *)(DAT_0001a220 + 4) + 0x16;
  uVar2 = 0;
  do {
    if (*(byte *)(*(int *)(DAT_0001a220 + 4) + 0x46) <= uVar2) {
LAB_0001a1aa:
      if (param_2 == 0) {
        bVar1 = *(byte *)(iVar3 + 2);
      }
      else {
        bVar1 = *(byte *)(iVar3 + 1);
      }
      return (int)(((uint)bVar1 * 4 + (int)*(short *)(DAT_0001a220 + 0x30)) * 0x10000) >> 0x10;
    }
    if (param_1 <= *(byte *)(iVar3 + uVar2 * 3)) {
      if ((uVar2 != 0) && (param_1 < *(byte *)(iVar3 + uVar2 * 3))) {
        uVar2 = uVar2 - 1 & 0xff;
      }
      iVar3 = uVar2 * 3 + iVar3;
      goto LAB_0001a1aa;
    }
    uVar2 = uVar2 + 1 & 0xff;
  } while( true );
}



/* ======================================================================
 * 0001a1be  phy_gain_slot_update_fields
 * ====================================================================== */

undefined8
phy_gain_slot_update_fields
          (undefined4 param_1,undefined4 param_2,undefined2 param_3,undefined4 param_4,
          undefined2 param_5,undefined2 param_6,undefined1 param_7,int param_8,uint param_9)

{
  int iVar1;
  int iVar2;
  
  iVar1 = DAT_0001a228;
  iVar2 = param_8 * 0x10 + DAT_0001a228;
  if ((param_9 & 1) != 0) {
    *(short *)(iVar2 + 2) = (short)param_1;
  }
  if ((int)(param_9 << 0x1e) < 0) {
    *(short *)(iVar2 + 4) = (short)param_2;
  }
  if ((int)(param_9 << 0x1d) < 0) {
    *(undefined2 *)(iVar2 + 6) = param_3;
  }
  if ((int)(param_9 << 0x1c) < 0) {
    *(undefined4 *)(iVar2 + 8) = param_4;
  }
  if ((int)(param_9 << 0x1b) < 0) {
    *(undefined2 *)(iVar2 + 0xc) = param_5;
  }
  if ((int)(param_9 << 0x1a) < 0) {
    *(undefined2 *)(iVar2 + 0xe) = param_6;
  }
  if ((int)(param_9 << 0x19) < 0) {
    *(undefined1 *)(iVar1 + param_8 * 0x10) = param_7;
  }
  return CONCAT44(param_2,param_1);
}



/* ======================================================================
 * 0001a250  rf_save_and_set_test_mode
 * ====================================================================== */

void rf_save_and_set_test_mode(void)

{
  undefined4 *puVar1;
  undefined4 *puVar2;
  int iVar3;
  uint uVar4;
  
  *(uint *)(DAT_0001a2a0 + 4) = *(uint *)(DAT_0001a2a0 + 4) | 0x800;
  iVar3 = DAT_0001a2a8;
  puVar1 = DAT_0001a2a4;
  *(undefined4 *)(DAT_0001a2a8 + 0x18) = DAT_0001a2a4[1];
  puVar1[1] = puVar1[6] | DAT_0001a2ac;
  puVar2 = DAT_0001a2a4;
  *(undefined4 *)(iVar3 + 0x1c) = DAT_0001a2a4[-5];
  puVar2[-5] = *puVar1;
  *(undefined4 *)(iVar3 + 0x58) = puVar2[-0xc];
  puVar2[-0xc] = puVar2[-7];
  puVar1 = DAT_0001a2a4;
  *(undefined4 *)(iVar3 + 0x54) = DAT_0001a2a4[-0x13];
  puVar1[-0x13] = puVar2[-0xe] | 0x101;
  uVar4 = puVar1[-0x1f];
  *(uint *)(iVar3 + 0x50) = uVar4;
  puVar1[-0x1f] = uVar4 | 2;
  return;
}



/* ======================================================================
 * 0001a448  bab_session_teardown
 * ====================================================================== */

void bab_session_teardown(int param_1)

{
  int iVar1;
  uint *flags;
  undefined1 *puVar2;
  int iVar3;
  
  clear_fields_718_and_20(param_1);
  iVar1 = DAT_0001a730;
  iVar3 = param_1 * 0x28 + DAT_0001a730 + -0xc0;
  if ((*(byte *)(DAT_0001a730 + 0x10) & 1) != 0) {
    puVar2 = (undefined1 *)lmc_msg_alloc();
    if (puVar2 != (undefined1 *)0x0) {
      puVar2[0x28] = *(undefined1 *)(iVar3 + 0x3ab);
      *puVar2 = 6;
      puVar2[0x29] = 8;
      fw_memcpy(puVar2 + 8,(void *)(iVar3 + 0x3a4),6);
      puVar2[4] = *(undefined1 *)(iVar3 + 0x3aa);
      *(undefined2 *)(puVar2 + 2) = 0x27;
      flags = DAT_0001a734;
      puVar2[1] = 0;
      evt_flags_set(flags,0x400000);
      pipe_clear_entry(param_1);
      timer_cancel(iVar3 + 0x3b4);
      *(undefined4 *)(iVar3 + 0x3a0) = 0;
      *(char *)(iVar1 + 0x12) = *(char *)(iVar1 + 0x12) + -1;
    }
    return;
  }
  ind_0808_ba_timeout(*(undefined1 *)(iVar3 + 0x3aa),(void *)(iVar3 + 0x3a4));
  return;
}



/* ======================================================================
 * 0001a4d6  bab_cancel_all_timers
 * ====================================================================== */

void bab_cancel_all_timers(void)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  iVar1 = DAT_0001a730;
  uVar3 = 0;
  iVar4 = DAT_0001a730 + -0xc0;
  *(undefined1 *)(DAT_0001a730 + 0x12) = 0;
  do {
    iVar2 = uVar3 * 0x28 + iVar4;
    *(undefined4 *)(iVar2 + 0x3a0) = 0;
    timer_cancel(iVar2 + 0x3b4);
    uVar3 = uVar3 + 1;
  } while (uVar3 < 4);
  link_state_init_all();
  *(undefined1 *)(iVar1 + 0x10) = 0;
  *(undefined2 *)(iVar1 + 0x18) = 0;
  *(undefined2 *)(iVar1 + 0x1c) = 0;
  *(undefined2 *)(iVar1 + 0x1a) = 0;
  *(undefined2 *)(iVar1 + 0x1e) = 0;
  *(undefined1 *)(iVar1 + 0x13) = 0;
  *(undefined1 *)(iVar1 + 0x14) = 0;
  return;
}



/* ======================================================================
 * 0001a516  bab_free_sessions_for_vif
 * ====================================================================== */

void bab_free_sessions_for_vif(uint param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  uVar2 = 0;
  iVar3 = DAT_0001a730 + -0xc0;
  do {
    iVar1 = uVar2 * 0x28 + iVar3;
    if (*(byte *)(iVar1 + 0x3ab) == param_1) {
      if (*(int *)(iVar1 + 0x3a0) != 0) {
        *(undefined4 *)(iVar1 + 0x3a0) = 0;
        timer_cancel(iVar1 + 0x3b4);
        pipe_clear_entry(uVar2);
        clear_field_718(uVar2);
        *(char *)(DAT_0001a730 + 0x12) = *(char *)(DAT_0001a730 + 0x12) + -1;
      }
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 4);
  link_remove_by_vif(param_1);
  iVar3 = param_1 * 2 + iVar3;
  *(undefined2 *)(iVar3 + 0xd8) = 0;
  *(undefined2 *)(iVar3 + 0xdc) = 0;
  return;
}



/* ======================================================================
 * 0001a578  bab_teardown_vif_sessions
 * ====================================================================== */

void bab_teardown_vif_sessions(uint param_1)

{
  int iVar1;
  uint uVar2;
  int iVar3;
  
  link_states_reset_to_1();
  uVar2 = 0;
  iVar3 = DAT_0001a730 + -0xc0;
  do {
    iVar1 = uVar2 * 0x28 + iVar3;
    if ((*(byte *)(iVar1 + 0x3ab) == param_1) && (*(int *)(iVar1 + 0x3a0) != 0)) {
      timer_cancel(iVar1 + 0x3b4);
      bab_session_teardown(uVar2);
      iVar1 = (uint)*(byte *)(DAT_0001a730 + 0x13) * 0x2c + DAT_0001a730;
      if ((*(char *)(iVar1 + 0x20) == '\x06') && (*(short *)(iVar1 + 0x22) == 0x27)) {
        *(undefined2 *)(iVar1 + 0x22) = 0x25;
      }
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 4);
  return;
}



/* ======================================================================
 * 0001a5da  bab_link_state_check
 * ====================================================================== */

undefined4 bab_link_state_check(int param_1,int param_2,int param_3)

{
  char cVar1;
  short sVar2;
  int iVar3;
  undefined1 *puVar4;
  char *pcVar5;
  int iVar6;
  undefined4 local_18;
  
  puVar4 = DAT_0001a748;
  iVar3 = DAT_0001a730;
  local_18 = 0;
  pcVar5 = (char *)(param_1 * 0x3b0 + DAT_0001a738 + 0x18);
  sVar2 = *(short *)(DAT_0001a740 + 0x14);
  if (*(char *)(DAT_0001a73c + 5) == '\0') {
    *(byte *)(DAT_0001a730 + 0x10) = *(byte *)(DAT_0001a730 + 0x10) | 1;
    iVar6 = param_1 * 2 + iVar3 + -0xc0;
    *(short *)(iVar6 + 0xd8) = (short)param_2;
    *(short *)(param_1 * 2 + DAT_0001a744 + 0x648) = (short)param_2;
    *(short *)(iVar6 + 0xdc) = (short)param_3;
    if ((param_2 == 0 && param_3 == 0) &&
       (((cVar1 = *pcVar5, cVar1 == '\x04' || (cVar1 == '\x06')) && (sVar2 == 2)))) {
      *(byte *)(iVar3 + 0x10) = *(byte *)(iVar3 + 0x10) | 1;
      bab_teardown_vif_sessions();
    }
  }
  else {
    local_18 = 1;
    if ((((param_2 == 0) && (param_3 == 0)) &&
        ((cVar1 = *pcVar5, cVar1 == '\x04' || (cVar1 == '\x06')))) && (sVar2 == 2)) {
      *DAT_0001a748 = 1;
      puVar4[1] = (char)param_1;
    }
  }
  return local_18;
}



/* ======================================================================
 * 0001a664  bab_should_buffer_for_link
 * ====================================================================== */

undefined4 bab_should_buffer_for_link(uint param_1,int param_2)

{
  uint uVar1;
  int iVar2;
  short *psVar3;
  uint uVar4;
  int iVar5;
  
  uVar4 = 0;
  uVar1 = 0;
  do {
    if (*(ushort *)(DAT_0001a740 + 0x14) <= uVar1) {
LAB_0001a6dc:
      if (((uVar4 != 0) && (uVar4 < 0xf)) &&
         ((((uint)*(ushort *)(param_1 * 0x3b0 + DAT_0001a738 + 0x15c) & 1 << uVar4) != 0 ||
          ((((uint)*(ushort *)(param_1 * 2 + DAT_0001a730 + 0x1c) &
            1 << ((*(byte *)(param_2 + 0x1b) & 0x3f) >> 2)) == 0 ||
           (*(int *)(DAT_0001a750 + 0x14) == 0)))))) {
        return 1;
      }
      return 0;
    }
    iVar2 = uVar1 * 0xc + DAT_0001a738;
    iVar5 = iVar2 + DAT_0001a74c + -0x20;
    if ((((*(short *)(iVar5 + 0x1e) == *(short *)(param_2 + 10)) &&
         (psVar3 = (short *)(iVar2 + DAT_0001a74c), *psVar3 == *(short *)(param_2 + 0xc))) &&
        (psVar3[1] == *(short *)(param_2 + 0xe))) && (*(byte *)(iVar5 + 0x19) == param_1)) {
      uVar4 = (uint)*(byte *)(uVar1 * 0xc + DAT_0001a738 + DAT_0001a74c + -8);
      goto LAB_0001a6dc;
    }
    uVar1 = uVar1 + 1 & 0xff;
  } while( true );
}



/* ======================================================================
 * 0001a71a  bab_session_try_alloc
 * ====================================================================== */

/* bab_session_try_alloc() -- claim one of the block-ack session slots.
   Returns 0 on success, 1 when full.
   
     if (g_bab->count < 4) { g_bab->count++; return 0; }
     return 1;
   
   **This is the firmware's hard 4-session block-ack limit**, matching what
   WSM 0x0014's return value already advertised (`4 - active_count`).  The slot
   table is 4 entries of stride 0x28 at g_bab - 0xC0, with per-entry fields:
     +0x3A0  in-use / inactivity seed (u32, non-zero = active)
     +0x3A4  peer MAC (6 bytes)
     +0x3AA  TID
     +0x3AB  if_id
     +0x3B2  timeout in units of 1024 us
     +0x3B4  timer
   
   Caller: rx_mgmt_frame_handler (0x0000C4F0), on a received ADDBA.  When the
   allocation fails it queues an internal message with code 0x26 carrying the
   peer MAC, i.e. the request is refused rather than dropped silently.
   
   *** NAMING CORRECTION: an earlier pass in this session named this group
   ap_inact_slot_try_alloc / ap_link_inactivity_expire, reading the MAC + timer +
   4-entry table as AP station inactivity tracking.  That was wrong.  The index
   comes from bab_find_session(tid) and the caller is the RX management path, so
   these are block-ack sessions.  AP inactivity is a separate mechanism, seeded in
   ap_map_link from vif+0x3A0/0x3A1 (MIB 0x1035). *** */

undefined4 bab_session_try_alloc(void)

{
  if (*(byte *)(DAT_0001a730 + 0x11) < 4) {
    *(byte *)(DAT_0001a730 + 0x11) = *(byte *)(DAT_0001a730 + 0x11) + 1;
    return 0;
  }
  return 1;
}



/* ======================================================================
 * 0001a754  lmc_msg_complete_dispatch
 * ====================================================================== */

void lmc_msg_complete_dispatch(int param_1)

{
  char cVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  cVar1 = *(char *)(param_1 + 0x22);
  if (cVar1 == '\x01') {
    if (*(int *)(DAT_0001a824 + 0x4c) == param_1) {
      *(undefined4 *)(DAT_0001a824 + 0x4c) = *(undefined4 *)(param_1 + 4);
    }
  }
  else {
    if (cVar1 != '\x04') {
      if (cVar1 != '\x03') goto LAB_0001a7b4;
      uVar3 = 0;
      do {
        iVar4 = uVar3 * 0x3b0 + DAT_0001a828;
        if ((*(char *)(iVar4 + 0x66) == '\x02') &&
           (*(short *)(iVar4 + 0x52) == *(short *)(param_1 + 0xe))) {
          *(undefined1 *)(iVar4 + 0x50) = 0x33;
          iVar2 = DAT_0001a824;
          *(undefined1 *)(iVar4 + 0x66) = 3;
          *(int *)(iVar2 + 0x48) = iVar4 + 0x44;
          goto LAB_0001a7b4;
        }
        uVar3 = uVar3 + 1;
      } while (uVar3 < 3);
    }
    lmc_sched_radio_release();
  }
LAB_0001a7b4:
  *(undefined1 *)(param_1 + 0x22) = 0;
  return;
}



/* ======================================================================
 * 0001a7d6  lmc_p2p_timer_restart
 * ====================================================================== */

void lmc_p2p_timer_restart(void)

{
  int iVar1;
  int iVar2;
  
  iVar1 = DAT_0001a82c;
  *DAT_0001a830 = 0;
  iVar2 = DAT_0001a824 + 0xa8;
  if (*(char *)(iVar1 + 1) != '\0') {
    if (*(char *)(iVar1 + 0x4e) == '\0') {
      *(undefined1 *)(iVar1 + 0x39) = 2;
      *(undefined2 *)(iVar1 + 0x3a) = *(undefined2 *)(iVar1 + 0x2a);
      *(undefined4 *)(iVar1 + 0x3c) = 0xffffffff;
      *(undefined4 *)(iVar1 + 0x44) = 0;
      *(undefined1 *)(iVar1 + 0x38) = 0x23;
      lmc_sched_request_radio(iVar1 + 0x2c);
    }
    timer_start(iVar2,&DAT_00005000);
    return;
  }
  timer_cancel(iVar2);
  return;
}



/* ======================================================================
 * 0001a834  beacon_schedule_next
 * ====================================================================== */

void beacon_schedule_next(int param_1)

{
  int iVar1;
  int iVar2;
  int iVar3;
  
  txp_program_duration();
  iVar2 = param_1 * 0x3b0 + DAT_0001a950;
  tsf_read(param_1);
  iVar3 = *(int *)(iVar2 + 0x118);
  iVar1 = iVar3;
  __udivmoddi4();
  iVar1 = (iVar3 - iVar1) + -4000;
  if (iVar1 < 1000) {
    iVar1 = iVar3 + iVar1;
  }
  timer_start(iVar2 + 0xd8,iVar1);
  evt_flags_set(DAT_0001a954,0x40);
  return;
}



/* ======================================================================
 * 0001a882  ap_scan_max_sta_byte
 * ====================================================================== */

void ap_scan_max_sta_byte(char *param_1)

{
  byte bVar1;
  byte bVar2;
  uint uVar3;
  
  bVar2 = 0;
  uVar3 = 0;
  if ((*param_1 == '\x04') || (*param_1 == '\x06')) {
    for (; uVar3 < *(ushort *)(DAT_0001a950 + DAT_0001a958 + 0x14); uVar3 = uVar3 + 1 & 0xff) {
      bVar1 = *(byte *)(uVar3 * 0xc + DAT_0001a950 + DAT_0001a958 + 0x18);
      if (bVar2 < bVar1) {
        bVar2 = bVar1;
      }
    }
  }
  return;
}



/* ======================================================================
 * 0001a8ba  beacon_fill_tim
 * ====================================================================== */

void beacon_fill_tim(int param_1)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  int iVar4;
  
  iVar4 = param_1 * 0x3b0 + DAT_0001a950;
  iVar2 = param_1 * 0x70 + DAT_0001a95c;
  iVar2 = ie_find_in_frame(*(undefined4 *)(iVar2 + 0x10),*(undefined2 *)(iVar2 + 0x18),5,0);
  if (iVar2 != 0) {
    *(undefined1 *)(iVar2 + 2) = *(undefined1 *)(iVar4 + 0x164);
    iVar1 = DAT_0001a960;
    if ((*(char *)(iVar4 + 0x3a1) != '\0') && (*(char *)(iVar4 + 0x3a0) != '\0')) {
      *(char *)(iVar2 + 5) = (char)*(undefined2 *)(DAT_0001a960 + 0x16);
      uVar3 = ap_scan_max_sta_byte(iVar4 + 0x18);
      if (7 < uVar3) {
        *(char *)(iVar2 + 6) = (char)((ushort)*(undefined2 *)(iVar1 + 0x16) >> 8);
      }
    }
  }
  return;
}



/* ======================================================================
 * 0001a91a  beacon_stop_for_vif
 * ====================================================================== */

void beacon_stop_for_vif(int param_1)

{
  int iVar1;
  uint uVar2;
  undefined4 uVar3;
  
  iVar1 = DAT_0001a950;
  timer_cancel(param_1 * 0x3b0 + DAT_0001a950 + 0xd8);
  uVar3 = 1;
  uVar2 = 0;
  do {
    if (*(int *)(uVar2 * 0x3b0 + iVar1 + 0x1c) << 0xe < 0) {
      uVar3 = 0;
    }
    uVar2 = uVar2 + 1;
  } while (uVar2 < 2);
  mac_disable_beacon_hw(param_1,uVar3);
  return;
}



/* ======================================================================
 * 0001a964  vif_set_slot_and_rate_state
 * ====================================================================== */

void vif_set_slot_and_rate_state
               (int param_1,int *param_2,uint param_3,uint param_4,undefined4 param_5)

{
  int iVar1;
  int iVar2;
  uint uVar3;
  uint local_1c;
  uint local_18;
  
  iVar1 = param_1 * 0x98 + DAT_0001aa80;
  local_1c = param_3;
  local_18 = param_4;
  pas_build_phy_rate_words(&local_18,&local_1c,param_5,8,param_4);
  *param_2 = (local_1c & 0xffffff) + 0x51000000;
  param_2[1] = (local_18 & 0xffffff) + 0x50000000;
  iVar2 = pas_rate_to_hw_code(param_5);
  uVar3 = iVar2 << 0x10 | DAT_0001aa84;
  param_2[3] = DAT_0001aa88;
  param_2[4] = 0x47000000;
  iVar2 = DAT_0001aa8c;
  param_2[2] = uVar3;
  param_2[5] = iVar2;
  param_2[6] = (param_3 & 0xffffff) + 0x32000000;
  param_2[7] = *(uint3 *)(iVar1 + 0x47c) + 0x33000000;
  param_2[8] = *(uint3 *)(iVar1 + 0x47f) + 0x33000000;
  param_2[9] = DAT_0001aa90;
  param_2[10] = -0x10000000;
  return;
}



/* ======================================================================
 * 0001aa0e  hw_dma_program_desc
 * ====================================================================== */

void hw_dma_program_desc(undefined4 param_1,uint *param_2,undefined4 param_3,undefined4 param_4,
                        short param_5,undefined4 param_6)

{
  int iVar1;
  
  iVar1 = DAT_0001aa94;
  *param_2 = *(uint *)(DAT_0001aa94 + 0x30) |
             (*(uint *)(DAT_0001aa94 + 0x30) & 0x1fff) << 0x10 | DAT_0001aa98;
  *(ushort *)(param_2 + 1) = ((ushort)*(undefined4 *)(iVar1 + 0x30) & 0x1fff) + 0x8000;
  *(short *)((int)param_2 + 6) = param_5 + -0x4400;
  param_2[2] = 0x18000000;
  vif_set_slot_and_rate_state(param_1,param_2 + 3,param_3,param_4,param_6);
  return;
}



/* ======================================================================
 * 0001aa52  hw_dma_kick
 * ====================================================================== */

void hw_dma_kick(undefined4 param_1,undefined4 param_2,undefined4 param_3,undefined4 param_4)

{
  uint uVar1;
  
  uVar1 = DAT_0001aa9c;
  hw_dma_program_desc(param_1,DAT_0001aa9c,param_2,param_4,1,param_3);
  *(uint *)(DAT_0001aaa0 + 0x34) = uVar1 & 0xf6ffffff;
  *DAT_0001aaa8 = *(uint *)(DAT_0001aaa4 + 0x30) | 0x10000;
  return;
}



/* ======================================================================
 * 0001aaac  s64_div
 * ====================================================================== */

int s64_div(int param_1,int param_2,int param_3,uint param_4)

{
  uint uVar1;
  uint uVar2;
  int iVar3;
  bool bVar4;
  
  uVar2 = param_2 >> 1;
  uVar1 = param_4 >> 1;
  if ((int)uVar2 < 0) {
    bVar4 = param_1 != 0;
    param_1 = -param_1;
    param_2 = -(param_2 + (uint)bVar4);
  }
  if ((int)param_4 < 0) {
    bVar4 = param_3 != 0;
    param_3 = -param_3;
    param_4 = -(param_4 + bVar4);
  }
  iVar3 = __udivmoddi4(param_1,param_2,param_3,param_4);
  if (((uVar2 ^ uVar1) & 0x40000000) != 0) {
    iVar3 = -iVar3;
  }
  return iVar3;
}



/* ======================================================================
 * 0001ab00  u64_shl
 * ====================================================================== */

longlong u64_shl(uint param_1,int param_2,uint param_3)

{
  if ((int)(param_3 - 0x20) < 0) {
    return CONCAT44(param_2 << (param_3 & 0xff) | param_1 >> (0x20 - param_3 & 0xff),
                    param_1 << (param_3 & 0xff));
  }
  return (ulonglong)(param_1 << (param_3 - 0x20 & 0xff)) << 0x20;
}



/* ======================================================================
 * 0001ab28  u64_mul_full
 * ====================================================================== */

undefined8 u64_mul_full(uint param_1,int param_2,uint param_3,int param_4)

{
  return CONCAT44(param_4 * param_1 +
                  param_3 * param_2 + (int)((ulonglong)param_1 * (ulonglong)param_3 >> 0x20),
                  (int)((ulonglong)param_1 * (ulonglong)param_3));
}



/* ======================================================================
 * 0001ab44  __udivmoddi4
 * ====================================================================== */

ulonglong __udivmoddi4(uint param_1,uint param_2,uint param_3,uint param_4)

{
  uint uVar1;
  undefined4 extraout_r1;
  undefined4 uVar2;
  uint uVar3;
  uint uVar4;
  uint uVar5;
  int iVar6;
  uint uVar7;
  uint uVar8;
  undefined4 *unaff_r5;
  uint uVar9;
  int iVar10;
  uint uVar11;
  undefined4 *puVar12;
  undefined4 uVar13;
  int in_lr;
  bool bVar14;
  bool bVar15;
  bool bVar16;
  bool bVar17;
  uint in_cpsr;
  undefined8 uVar18;
  uint uStack00000010;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  uint uStack_18;
  uint uStack_14;
  
  uStack00000010 = param_4 | param_3 >> 0x1f;
  if (uStack00000010 != 0) {
    if ((param_4 & 0x80000000) != 0) {
      return (ulonglong)(param_4 < param_2 || param_2 - param_4 < (uint)(param_3 <= param_1));
    }
    uVar11 = LZCOUNT(param_4) - 1;
    uVar9 = param_3 << (uVar11 & 0xff);
    uVar4 = param_4 << (uVar11 & 0xff) | param_3 >> (0x20 - uVar11 & 0xff);
    bVar14 = param_2 - uVar4 < (uint)(uVar9 <= param_1);
    uVar1 = param_2;
    if (uVar4 < param_2 || bVar14) {
      uVar1 = param_2 - (uVar4 + (uVar9 > param_1));
      param_1 = param_1 - uVar9;
    }
    bVar15 = uVar1 - uVar4 < (uint)(uVar9 <= param_1);
    uVar3 = uVar1;
    if (uVar4 < uVar1 || bVar15) {
      uVar3 = uVar1 - (uVar4 + (uVar9 > param_1));
      param_1 = param_1 - uVar9;
    }
    bVar17 = uVar3 - uVar4 < (uint)(uVar9 <= param_1);
    uVar7 = param_1;
    uVar8 = uVar3;
    if (uVar4 < uVar3 || bVar17) {
      uVar7 = param_1 - uVar9;
      uVar8 = uVar3 - (uVar4 + (uVar9 > param_1));
    }
    uVar3 = (uint)(uVar4 < param_2 || bVar14) + (uint)(uVar4 < uVar1 || bVar15) +
            (uint)(uVar4 < uVar3 || bVar17);
    uVar1 = -uVar9;
    uVar9 = -(uVar4 + (uVar9 != 0));
    uVar4 = LZCOUNT(uVar8) - 2;
    if ((int)uVar4 < 0) {
      uVar4 = 0;
    }
    if (uVar11 < uVar4) {
      uVar4 = uVar11;
    }
    uVar8 = uVar8 << (uVar4 & 0xff) | uVar7 >> (0x20 - uVar4 & 0xff);
    uVar7 = uVar7 << (uVar4 & 0xff);
    bVar14 = false;
    if (uVar11 - uVar4 != 0) {
      uVar5 = (uVar11 - uVar4) - 1;
      uVar4 = 3 - (uVar5 & 3);
      bVar14 = CARRY4(uVar4,uVar4 * 2);
      switch(uVar5 & 3) {
      case 1:
        goto switchD_0001ad3c_caseD_1;
      case 2:
        goto switchD_0001ad3c_caseD_2;
      case 3:
        goto switchD_0001ad3c_caseD_3;
      }
      while( true ) {
        bVar15 = CARRY4(uVar7,uVar7) || CARRY4(uVar7 * 2,(uint)bVar14);
        uVar7 = uVar7 * 2 + (uint)bVar14;
        bVar17 = CARRY4(uVar8,uVar8) || CARRY4(uVar8 * 2,(uint)bVar15);
        uVar8 = uVar8 * 2 + (uint)bVar15;
        bVar16 = CARRY4(uVar1,uVar7) || CARRY4(uVar1 + uVar7,(uint)bVar17);
        bVar15 = CARRY4(uVar9 + uVar8,(uint)bVar16);
        bVar14 = CARRY4(uVar9,uVar8) || bVar15;
        uVar5 = uVar5 - 4;
        if (CARRY4(uVar9,uVar8) || bVar15) {
          uVar8 = uVar9 + uVar8 + (uint)bVar16;
          uVar7 = uVar1 + uVar7 + (uint)bVar17;
        }
        if ((int)uVar5 < 0) break;
switchD_0001ad3c_caseD_3:
        bVar15 = CARRY4(uVar7,uVar7) || CARRY4(uVar7 * 2,(uint)bVar14);
        uVar7 = uVar7 * 2 + (uint)bVar14;
        bVar17 = CARRY4(uVar8,uVar8) || CARRY4(uVar8 * 2,(uint)bVar15);
        uVar8 = uVar8 * 2 + (uint)bVar15;
        bVar16 = CARRY4(uVar1,uVar7) || CARRY4(uVar1 + uVar7,(uint)bVar17);
        bVar15 = CARRY4(uVar9 + uVar8,(uint)bVar16);
        bVar14 = CARRY4(uVar9,uVar8) || bVar15;
        if (CARRY4(uVar9,uVar8) || bVar15) {
          uVar8 = uVar9 + uVar8 + (uint)bVar16;
          uVar7 = uVar1 + uVar7 + (uint)bVar17;
        }
switchD_0001ad3c_caseD_2:
        bVar15 = CARRY4(uVar7,uVar7) || CARRY4(uVar7 * 2,(uint)bVar14);
        uVar7 = uVar7 * 2 + (uint)bVar14;
        bVar17 = CARRY4(uVar8,uVar8) || CARRY4(uVar8 * 2,(uint)bVar15);
        uVar8 = uVar8 * 2 + (uint)bVar15;
        bVar16 = CARRY4(uVar1,uVar7) || CARRY4(uVar1 + uVar7,(uint)bVar17);
        bVar15 = CARRY4(uVar9 + uVar8,(uint)bVar16);
        bVar14 = CARRY4(uVar9,uVar8) || bVar15;
        if (CARRY4(uVar9,uVar8) || bVar15) {
          uVar8 = uVar9 + uVar8 + (uint)bVar16;
          uVar7 = uVar1 + uVar7 + (uint)bVar17;
        }
switchD_0001ad3c_caseD_1:
        bVar15 = CARRY4(uVar7,uVar7) || CARRY4(uVar7 * 2,(uint)bVar14);
        uVar7 = uVar7 * 2 + (uint)bVar14;
        bVar17 = CARRY4(uVar8,uVar8) || CARRY4(uVar8 * 2,(uint)bVar15);
        uVar8 = uVar8 * 2 + (uint)bVar15;
        bVar16 = CARRY4(uVar1,uVar7) || CARRY4(uVar1 + uVar7,(uint)bVar17);
        bVar15 = CARRY4(uVar9 + uVar8,(uint)bVar16);
        bVar14 = CARRY4(uVar9,uVar8) || bVar15;
        if (CARRY4(uVar9,uVar8) || bVar15) {
          uVar8 = uVar9 + uVar8 + (uint)bVar16;
          uVar7 = uVar1 + uVar7 + (uint)bVar17;
        }
      }
    }
    uVar7 = uVar7 & ~((uVar7 >> (uVar11 & 0xff)) << (uVar11 & 0xff));
    bVar15 = CARRY4(uVar7,uVar7) || CARRY4(uVar7 * 2,(uint)bVar14);
    uVar1 = uVar7 * 2 + (uint)bVar14;
    uVar11 = uVar11 & 0xff;
    uVar9 = uVar3 << uVar11;
    return CONCAT44((uint)bVar15 +
                    (uint)(uVar11 == 0 && bVar15 ||
                          uVar11 != 0 && (bool)((byte)(uVar3 >> 0x20 - uVar11) & 1)) +
                    (uint)CARRY4(uVar1,uVar9),uVar1 + uVar9);
  }
  if (param_3 != 0) {
    iVar10 = LZCOUNT(param_3);
    uVar11 = iVar10 - 1;
    param_3 = param_3 << (uVar11 & 0xff);
    iVar6 = 0;
    if (param_3 << 1 <= param_2) {
      iVar6 = 2;
      param_2 = param_2 + param_3 * -2;
    }
    bVar14 = param_3 <= param_2;
    if (bVar14) {
      param_2 = param_2 - param_3;
    }
    uVar4 = iVar10 + 0x1f;
    uVar1 = -param_3;
    uVar9 = param_1;
    if ((0x1f < uVar4) && (param_2 == 0 && param_1 >> 0x1e == 0)) {
      uVar9 = 0;
      uVar4 = iVar10 - 1;
      param_2 = param_1;
    }
    uVar3 = LZCOUNT(param_2) - 2;
    if ((int)uVar3 < 0) {
      uVar3 = 0;
    }
    if (uVar4 < uVar3) {
      uVar3 = uVar4;
    }
    uVar7 = param_2 << (uVar3 & 0xff) | uVar9 >> (0x20 - uVar3 & 0xff);
    uVar9 = uVar9 << (uVar3 & 0xff);
    bVar15 = false;
    if (uVar4 - uVar3 != 0) {
      uVar3 = (uVar4 - uVar3) - 1;
      uVar4 = uVar3 & 7 ^ 7;
      bVar15 = CARRY4(uVar4,uVar4 * 2);
      switch(uVar3 & 7) {
      case 0:
        while( true ) {
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          uVar3 = uVar3 - 8;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
          if ((int)uVar3 < 0) break;
switchD_0001ac20_caseD_7:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
switchD_0001ac20_caseD_6:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
switchD_0001ac20_caseD_5:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
switchD_0001ac20_caseD_4:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
switchD_0001ac20_caseD_3:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
switchD_0001ac20_caseD_2:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
switchD_0001ac20_caseD_1:
          bVar16 = CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15);
          uVar9 = uVar9 * 2 + (uint)bVar15;
          uVar4 = uVar7 * 2;
          bVar17 = CARRY4(uVar1 + uVar4,(uint)bVar16);
          bVar15 = CARRY4(uVar1,uVar4) || bVar17;
          uVar7 = uVar1 + uVar4 + (uint)bVar16;
          if (!CARRY4(uVar1,uVar4) && !bVar17) {
            uVar7 = uVar7 + param_3;
          }
        }
        break;
      case 1:
        goto switchD_0001ac20_caseD_1;
      case 2:
        goto switchD_0001ac20_caseD_2;
      case 3:
        goto switchD_0001ac20_caseD_3;
      case 4:
        goto switchD_0001ac20_caseD_4;
      case 5:
        goto switchD_0001ac20_caseD_5;
      case 6:
        goto switchD_0001ac20_caseD_6;
      case 7:
        goto switchD_0001ac20_caseD_7;
      }
    }
    return CONCAT44((uVar7 & ~((uVar7 >> (uVar11 & 0xff)) << (uVar11 & 0xff))) * 2 +
                    (uint)(CARRY4(uVar9,uVar9) || CARRY4(uVar9 * 2,(uint)bVar15)) +
                    (iVar6 + (uint)bVar14 << (uVar11 & 0xff)),uVar9 * 2 + (uint)bVar15);
  }
  uStack_1c = 2;
  puStack00000014 = &uStack_20;
  uStack_20 = 2;
  iStack0000001c = in_lr + -4;
  bVar14 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar12 = &uStack_24;
  uStack_24 = 0;
  uVar13 = 0x16650;
  uStack_18 = param_3;
  uStack_14 = param_4;
  uVar18 = exc_build_indication_and_spin(puVar12);
  iVar6 = DAT_00016680;
  uVar2 = (undefined4)((ulonglong)uVar18 >> 0x20);
  iVar10 = (int)uVar18;
  if (bVar14) {
    uVar13 = *unaff_r5;
    puVar12 = (undefined4 *)unaff_r5[-1];
    iVar10 = unaff_r5[-5];
  }
  puVar12[-1] = uVar13;
  puVar12[-2] = 0;
  if (iVar10 == 0) {
    iVar10 = 0x10;
  }
  else {
    iVar10 = *(int *)(DAT_00016680 + 0x24) << 0x1f;
    if (iVar10 != 0) goto LAB_00016670;
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    iVar10 = 1;
    uVar2 = extraout_r1;
  }
  *(int *)(iVar6 + 0x24) = iVar10;
LAB_00016670:
  return CONCAT44(uVar2,iVar10);
}



/* ======================================================================
 * 0001ae10  memcpy_fast
 * ====================================================================== */

undefined8 memcpy_fast(uint *param_1,uint *param_2,uint param_3,uint param_4)

{
  uint *puVar1;
  uint *puVar2;
  byte *pbVar3;
  byte bVar4;
  byte bVar5;
  uint in_r12;
  uint uVar6;
  uint uVar7;
  uint uVar8;
  bool bVar9;
  bool bVar10;
  
  if (3 < param_3) {
    uVar6 = (uint)param_1 & 3;
    in_r12 = uVar6;
    if (uVar6 != 0) {
      bVar4 = (byte)*param_2;
      puVar2 = (uint *)((int)param_2 + 1);
      if (uVar6 < 3) {
        puVar2 = (uint *)((int)param_2 + 2);
        in_r12 = (uint)*(byte *)((int)param_2 + 1);
      }
      *(byte *)param_1 = bVar4;
      param_2 = puVar2;
      if (uVar6 < 2) {
        param_2 = (uint *)((int)puVar2 + 1);
        bVar4 = (byte)*puVar2;
      }
      puVar2 = (uint *)((int)param_1 + 1);
      if (uVar6 < 3) {
        puVar2 = (uint *)((int)param_1 + 2);
        *(byte *)((int)param_1 + 1) = (byte)in_r12;
      }
      param_3 = (param_3 + uVar6) - 4;
      param_1 = puVar2;
      if (uVar6 < 2) {
        param_1 = (uint *)((int)puVar2 + 1);
        *(byte *)puVar2 = bVar4;
      }
    }
    param_4 = (uint)param_2 & 3;
    if (param_4 == 0) {
      bVar9 = 0x1f < param_3;
      param_3 = param_3 - 0x20;
      if (bVar9) {
        do {
          if (bVar9) {
            uVar6 = param_2[1];
            uVar7 = param_2[2];
            uVar8 = param_2[3];
            *param_1 = *param_2;
            param_1[1] = uVar6;
            param_1[2] = uVar7;
            param_1[3] = uVar8;
            param_4 = param_2[4];
            uVar6 = param_2[5];
            uVar7 = param_2[6];
            uVar8 = param_2[7];
            param_2 = param_2 + 8;
            param_1[4] = param_4;
            param_1[5] = uVar6;
            param_1[6] = uVar7;
            param_1[7] = uVar8;
            param_1 = param_1 + 8;
            bVar9 = 0x1f < param_3;
            param_3 = param_3 - 0x20;
          }
        } while (bVar9);
      }
      puVar2 = param_1;
      if ((bool)((byte)(param_3 >> 4) & 1)) {
        param_4 = *param_2;
        uVar6 = param_2[1];
        uVar7 = param_2[2];
        uVar8 = param_2[3];
        param_2 = param_2 + 4;
        *param_1 = param_4;
        param_1[1] = uVar6;
        param_1[2] = uVar7;
        param_1[3] = uVar8;
        puVar2 = param_1 + 4;
      }
      if ((int)(param_3 << 0x1c) < 0) {
        param_4 = *param_2;
        uVar6 = param_2[1];
        param_2 = param_2 + 2;
        *puVar2 = param_4;
        puVar2[1] = uVar6;
        puVar2 = puVar2 + 2;
      }
      in_r12 = param_3 << 0x1e;
      param_1 = puVar2;
      if ((bool)((byte)(param_3 >> 2) & 1)) {
        param_4 = *param_2;
        param_1 = puVar2 + 1;
        *puVar2 = param_4;
        param_2 = param_2 + 1;
      }
      if (in_r12 == 0) {
        return CONCAT44(param_2,param_1);
      }
    }
    else {
      bVar9 = 3 < param_3;
      param_3 = param_3 - 4;
      if (bVar9) {
        param_2 = (uint *)((int)param_2 - param_4);
        in_r12 = *param_2;
        puVar2 = param_1;
        if (param_4 == 2) {
          do {
            puVar1 = param_2;
            param_4 = in_r12 >> 0x10;
            param_2 = puVar1 + 1;
            in_r12 = *param_2;
            bVar9 = 3 < param_3;
            param_3 = param_3 - 4;
            param_4 = param_4 | in_r12 << 0x10;
            param_1 = puVar2 + 1;
            *puVar2 = param_4;
            puVar2 = param_1;
          } while (bVar9);
          param_2 = (uint *)((int)puVar1 + 6);
        }
        else if (param_4 < 3) {
          do {
            puVar1 = param_2;
            param_4 = in_r12 >> 8;
            param_2 = puVar1 + 1;
            in_r12 = *param_2;
            bVar9 = 3 < param_3;
            param_3 = param_3 - 4;
            param_4 = param_4 | in_r12 << 0x18;
            param_1 = puVar2 + 1;
            *puVar2 = param_4;
            puVar2 = param_1;
          } while (bVar9);
          param_2 = (uint *)((int)puVar1 + 5);
        }
        else {
          do {
            puVar1 = param_2;
            param_4 = in_r12 >> 0x18;
            param_2 = puVar1 + 1;
            in_r12 = *param_2;
            bVar9 = 3 < param_3;
            param_3 = param_3 - 4;
            param_4 = param_4 | in_r12 << 8;
            param_1 = puVar2 + 1;
            *puVar2 = param_4;
            puVar2 = param_1;
          } while (bVar9);
          param_2 = (uint *)((int)puVar1 + 7);
        }
      }
    }
  }
  bVar5 = (byte)in_r12;
  bVar4 = (byte)param_4;
  bVar10 = (bool)((byte)(param_3 >> 1) & 1);
  param_3 = param_3 << 0x1f;
  bVar9 = (int)param_3 < 0;
  puVar2 = param_2;
  if (bVar9) {
    puVar2 = (uint *)((int)param_2 + 1);
    param_3 = (uint)(byte)*param_2;
  }
  if (bVar10) {
    pbVar3 = (byte *)((int)puVar2 + 1);
    bVar4 = (byte)*puVar2;
    puVar2 = (uint *)((int)puVar2 + 2);
    bVar5 = *pbVar3;
  }
  puVar1 = param_1;
  if (bVar9) {
    puVar1 = (uint *)((int)param_1 + 1);
    *(byte *)param_1 = (byte)param_3;
  }
  if (bVar10) {
    pbVar3 = (byte *)((int)puVar1 + 1);
    *(byte *)puVar1 = bVar4;
    puVar1 = (uint *)((int)puVar1 + 2);
    *pbVar3 = bVar5;
  }
  return CONCAT44(puVar2,puVar1);
}



/* ======================================================================
 * 0001aec4  fw_memcpy_bytes_ret
 * ====================================================================== */

undefined8 fw_memcpy_bytes_ret(byte *param_1,byte *param_2,uint param_3,undefined4 param_4)

{
  byte bVar1;
  byte *pbVar2;
  byte *pbVar3;
  byte *pbVar4;
  undefined4 uVar5;
  undefined4 uVar6;
  uint uVar7;
  undefined4 uVar8;
  bool bVar9;
  bool bVar10;
  
  bVar9 = 0x1f < param_3;
  param_3 = param_3 - 0x20;
  if (bVar9) {
    do {
      if (bVar9) {
        uVar5 = *(undefined4 *)(param_2 + 4);
        uVar6 = *(undefined4 *)(param_2 + 8);
        uVar8 = *(undefined4 *)(param_2 + 0xc);
        *(undefined4 *)param_1 = *(undefined4 *)param_2;
        *(undefined4 *)(param_1 + 4) = uVar5;
        *(undefined4 *)(param_1 + 8) = uVar6;
        *(undefined4 *)(param_1 + 0xc) = uVar8;
        param_4 = *(undefined4 *)(param_2 + 0x10);
        uVar5 = *(undefined4 *)(param_2 + 0x14);
        uVar6 = *(undefined4 *)(param_2 + 0x18);
        uVar8 = *(undefined4 *)(param_2 + 0x1c);
        param_2 = param_2 + 0x20;
        *(undefined4 *)(param_1 + 0x10) = param_4;
        *(undefined4 *)(param_1 + 0x14) = uVar5;
        *(undefined4 *)(param_1 + 0x18) = uVar6;
        *(undefined4 *)(param_1 + 0x1c) = uVar8;
        param_1 = param_1 + 0x20;
        bVar9 = 0x1f < param_3;
        param_3 = param_3 - 0x20;
      }
    } while (bVar9);
  }
  if ((bool)((byte)(param_3 >> 4) & 1)) {
    param_4 = *(undefined4 *)param_2;
    uVar5 = *(undefined4 *)(param_2 + 4);
    uVar6 = *(undefined4 *)(param_2 + 8);
    uVar8 = *(undefined4 *)(param_2 + 0xc);
    param_2 = param_2 + 0x10;
    *(undefined4 *)param_1 = param_4;
    *(undefined4 *)(param_1 + 4) = uVar5;
    *(undefined4 *)(param_1 + 8) = uVar6;
    *(undefined4 *)(param_1 + 0xc) = uVar8;
    param_1 = param_1 + 0x10;
  }
  if ((int)(param_3 << 0x1c) < 0) {
    param_4 = *(undefined4 *)param_2;
    uVar5 = *(undefined4 *)(param_2 + 4);
    param_2 = param_2 + 8;
    *(undefined4 *)param_1 = param_4;
    *(undefined4 *)(param_1 + 4) = uVar5;
    param_1 = param_1 + 8;
  }
  uVar7 = param_3 << 0x1e;
  pbVar2 = param_1;
  pbVar3 = param_2;
  if ((bool)((byte)(param_3 >> 2) & 1)) {
    pbVar3 = param_2 + 4;
    param_4 = *(undefined4 *)param_2;
    pbVar2 = param_1 + 4;
    *(undefined4 *)param_1 = param_4;
  }
  bVar1 = (byte)param_4;
  if (uVar7 != 0) {
    bVar10 = (bool)((byte)(param_3 >> 1) & 1);
    param_3 = param_3 << 0x1f;
    bVar9 = (int)param_3 < 0;
    pbVar4 = pbVar3;
    if (bVar9) {
      pbVar4 = pbVar3 + 1;
      param_3 = (uint)*pbVar3;
    }
    if (bVar10) {
      pbVar3 = pbVar4 + 1;
      bVar1 = *pbVar4;
      pbVar4 = pbVar4 + 2;
      uVar7 = (uint)*pbVar3;
    }
    pbVar3 = pbVar2;
    if (bVar9) {
      pbVar3 = pbVar2 + 1;
      *pbVar2 = (byte)param_3;
    }
    if (bVar10) {
      pbVar2 = pbVar3 + 1;
      *pbVar3 = bVar1;
      pbVar3 = pbVar3 + 2;
      *pbVar2 = (byte)uVar7;
    }
    return CONCAT44(pbVar4,pbVar3);
  }
  return CONCAT44(pbVar3,pbVar2);
}



/* ======================================================================
 * 0001af30  bzero_fast
 * ====================================================================== */

undefined4 * bzero_fast(undefined4 *param_1,uint param_2)

{
  uint uVar1;
  undefined1 *puVar2;
  undefined4 *puVar3;
  undefined4 *puVar4;
  bool bVar5;
  
  bVar5 = 0x1f < param_2;
  param_2 = param_2 - 0x20;
  do {
    if (bVar5) {
      *param_1 = 0;
      param_1[1] = 0;
      param_1[2] = 0;
      param_1[3] = 0;
      param_1[4] = 0;
      param_1[5] = 0;
      param_1[6] = 0;
      param_1[7] = 0;
      param_1 = param_1 + 8;
      bVar5 = 0x1f < param_2;
      param_2 = param_2 - 0x20;
    }
  } while (bVar5);
  if ((bool)((byte)(param_2 >> 4) & 1)) {
    *param_1 = 0;
    param_1[1] = 0;
    param_1[2] = 0;
    param_1[3] = 0;
    param_1 = param_1 + 4;
  }
  if ((int)(param_2 << 0x1c) < 0) {
    *param_1 = 0;
    param_1[1] = 0;
    param_1 = param_1 + 2;
  }
  uVar1 = param_2 << 0x1e;
  puVar3 = param_1;
  if ((bool)((byte)((param_2 << 0x1c) >> 0x1e) & 1)) {
    puVar3 = param_1 + 1;
    *param_1 = 0;
  }
  if (uVar1 != 0) {
    if ((int)uVar1 < 0) {
      puVar2 = (undefined1 *)((int)puVar3 + 1);
      *(undefined1 *)puVar3 = 0;
      puVar3 = (undefined4 *)((int)puVar3 + 2);
      *puVar2 = 0;
    }
    puVar4 = puVar3;
    if ((uVar1 & 0x40000000) != 0) {
      puVar4 = (undefined4 *)((int)puVar3 + 1);
      *(undefined1 *)puVar3 = 0;
    }
    return puVar4;
  }
  return puVar3;
}



/* ======================================================================
 * 0001af88  fw_div_scaled
 * ====================================================================== */

undefined8 fw_div_scaled(uint param_1,uint param_2)

{
  int iVar1;
  undefined4 extraout_r1;
  undefined4 uVar2;
  int iVar3;
  uint uVar4;
  undefined4 *unaff_r5;
  undefined4 *puVar5;
  undefined4 uVar6;
  int in_lr;
  bool bVar7;
  bool bVar8;
  bool bVar9;
  bool bVar10;
  uint in_cpsr;
  undefined8 uVar11;
  int iStack00000010;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  uint uStack_18;
  uint uStack_14;
  
  if (-1 < (int)(param_1 | param_2)) {
    iVar3 = 0;
    if (param_1 >> 1 < param_2) goto LAB_0001b008;
    if (param_2 <= param_1 >> 4) {
      if (param_2 <= param_1 >> 8) {
        uStack_14 = 0;
        uVar4 = param_2;
        goto LAB_0001b038;
      }
      bVar7 = param_2 <= param_1 >> 7;
      if (bVar7) {
        param_1 = param_1 + param_2 * -0x80;
      }
      bVar8 = param_2 <= param_1 >> 6;
      if (bVar8) {
        param_1 = param_1 + param_2 * -0x40;
      }
      bVar9 = param_2 <= param_1 >> 5;
      if (bVar9) {
        param_1 = param_1 + param_2 * -0x20;
      }
      bVar10 = param_2 <= param_1 >> 4;
      if (bVar10) {
        param_1 = param_1 + param_2 * -0x10;
      }
      iVar3 = (((uint)bVar7 * 2 + (uint)bVar8) * 2 + (uint)bVar9) * 2 + (uint)bVar10;
    }
    bVar7 = param_2 <= param_1 >> 3;
    if (bVar7) {
      param_1 = param_1 + param_2 * -8;
    }
    bVar8 = param_2 <= param_1 >> 2;
    if (bVar8) {
      param_1 = param_1 + param_2 * -4;
    }
    bVar9 = param_2 <= param_1 >> 1;
    if (bVar9) {
      param_1 = param_1 + param_2 * -2;
    }
    iVar3 = ((iVar3 * 2 + (uint)bVar7) * 2 + (uint)bVar8) * 2 + (uint)bVar9;
LAB_0001b008:
    uVar4 = param_1 - param_2;
    if (param_2 > param_1) {
      uVar4 = param_1;
    }
    return CONCAT44(uVar4,iVar3 * 2 + (uint)(param_2 <= param_1));
  }
  uStack_18 = param_2 & 0x80000000;
  if ((int)uStack_18 < 0) {
    param_2 = -param_2;
  }
  uStack_14 = uStack_18 ^ (int)param_1 >> 0x20;
  if (SUB41(param_1 >> 0x1f,0)) {
    param_1 = -param_1;
  }
  if (param_1 >> 4 < param_2) goto LAB_0001b0a8;
  uVar4 = param_2;
  if (param_1 >> 8 < param_2) goto LAB_0001b078;
LAB_0001b038:
  param_2 = uVar4 << 6;
  uStack_18 = 0xfc000000;
  if (param_1 >> 8 < param_2) goto LAB_0001b078;
  param_2 = uVar4 << 0xc;
  uStack_18 = 0xfff00000;
  if (param_1 >> 8 < param_2) goto LAB_0001b078;
  param_2 = uVar4 << 0x12;
  uStack_18 = 0xffffc000;
  if (param_2 <= param_1 >> 8) {
    uStack_18 = 0xffffff00;
    param_2 = uVar4 << 0x18;
  }
  bVar7 = param_2 == 0;
  iStack00000010 = -param_2;
  if (!bVar7) {
    do {
      if (bVar7) {
        param_2 = param_2 >> 6;
      }
LAB_0001b078:
      bVar7 = param_2 <= param_1 >> 7;
      if (bVar7) {
        param_1 = param_1 + param_2 * -0x80;
      }
      bVar8 = param_2 <= param_1 >> 6;
      if (bVar8) {
        param_1 = param_1 + param_2 * -0x40;
      }
      bVar9 = param_2 <= param_1 >> 5;
      if (bVar9) {
        param_1 = param_1 + param_2 * -0x20;
      }
      bVar10 = param_2 <= param_1 >> 4;
      if (bVar10) {
        param_1 = param_1 + param_2 * -0x10;
      }
      uStack_18 = (((uStack_18 * 2 + (uint)bVar7) * 2 + (uint)bVar8) * 2 + (uint)bVar9) * 2 +
                  (uint)bVar10;
LAB_0001b0a8:
      bVar7 = param_2 <= param_1 >> 3;
      if (bVar7) {
        param_1 = param_1 + param_2 * -8;
      }
      uVar4 = uStack_18 * 2 + (uint)bVar7;
      bVar8 = param_2 <= param_1 >> 2;
      if (bVar8) {
        param_1 = param_1 + param_2 * -4;
      }
      uStack_18 = uVar4 * 2 + (uint)bVar8;
      bVar7 = true;
    } while (CARRY4(uVar4,uVar4) || CARRY4(uVar4 * 2,(uint)bVar8));
    bVar7 = param_2 <= param_1 >> 1;
    if (bVar7) {
      param_1 = param_1 + param_2 * -2;
    }
    uVar4 = param_1 - param_2;
    if (param_2 > param_1) {
      uVar4 = param_1;
    }
    iVar3 = (uStack_18 * 2 + (uint)bVar7) * 2 + (uint)(param_2 <= param_1);
    if ((int)uStack_14 >> 0x1f < 0) {
      iVar3 = -iVar3;
    }
    if ((bool)((byte)(uStack_14 >> 0x1e) & 1)) {
      uVar4 = -uVar4;
    }
    return CONCAT44(uVar4,iVar3);
  }
  uStack_1c = 2;
  puStack00000014 = &uStack_20;
  uStack_20 = 2;
  iStack0000001c = in_lr + -4;
  bVar7 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar5 = &uStack_24;
  uStack_24 = 0;
  uVar6 = 0x16650;
  uVar11 = exc_build_indication_and_spin(puVar5);
  iVar3 = DAT_00016680;
  uVar2 = (undefined4)((ulonglong)uVar11 >> 0x20);
  iVar1 = (int)uVar11;
  if (bVar7) {
    uVar6 = *unaff_r5;
    puVar5 = (undefined4 *)unaff_r5[-1];
    iVar1 = unaff_r5[-5];
  }
  puVar5[-1] = uVar6;
  puVar5[-2] = 0;
  if (iVar1 == 0) {
    iVar1 = 0x10;
  }
  else {
    iVar1 = *(int *)(DAT_00016680 + 0x24) << 0x1f;
    if (iVar1 != 0) goto LAB_00016670;
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    iVar1 = 1;
    uVar2 = extraout_r1;
  }
  *(int *)(iVar3 + 0x24) = iVar1;
LAB_00016670:
  return CONCAT44(uVar2,iVar1);
}



/* ======================================================================
 * 0001b0ec  u64_rsub
 * ====================================================================== */

undefined8 u64_rsub(uint param_1,int param_2,uint param_3,int param_4)

{
  return CONCAT44(param_4 - (param_2 + (uint)(param_3 < param_1)),param_3 - param_1);
}



/* ======================================================================
 * 0001b0f8  s64_add_s32
 * ====================================================================== */

undefined8 s64_add_s32(uint param_1,int param_2,uint param_3)

{
  return CONCAT44(param_2 + ((int)param_3 >> 0x1f) + (uint)CARRY4(param_1,param_3),param_1 + param_3
                 );
}



/* ======================================================================
 * 0001b104  s64_sub_s32
 * ====================================================================== */

undefined8 s64_sub_s32(uint param_1,int param_2,uint param_3)

{
  return CONCAT44(param_2 - (((int)param_3 >> 0x1f) + (uint)(param_1 < param_3)),param_1 - param_3);
}



/* ======================================================================
 * 0001b110  u64_add_u32
 * ====================================================================== */

undefined8 u64_add_u32(uint param_1,int param_2,uint param_3)

{
  return CONCAT44(param_2 + (uint)CARRY4(param_1,param_3),param_1 + param_3);
}



/* ======================================================================
 * 0001b11c  u64_sub_u32
 * ====================================================================== */

undefined8 u64_sub_u32(uint param_1,int param_2,uint param_3)

{
  return CONCAT44(param_2 - (uint)(param_1 < param_3),param_1 - param_3);
}



/* ======================================================================
 * 0001b128  u64_mul_acc_u32
 * ====================================================================== */

undefined8 u64_mul_acc_u32(uint param_1,int param_2,uint param_3)

{
  return CONCAT44(param_3 * param_2 + (int)((ulonglong)param_1 * (ulonglong)param_3 >> 0x20),
                  (int)((ulonglong)param_1 * (ulonglong)param_3));
}



/* ======================================================================
 * 0001b130  u64_cmp
 * ====================================================================== */

void u64_cmp(void)

{
  return;
}



/* ======================================================================
 * 0001b13c  u64_sub
 * ====================================================================== */

undefined8 u64_sub(uint param_1,int param_2,uint param_3,int param_4)

{
  return CONCAT44(param_2 - (param_4 + (uint)(param_1 < param_3)),param_1 - param_3);
}



/* ======================================================================
 * 0001b148  u64_rsub_full
 * ====================================================================== */

undefined8 u64_rsub_full(uint param_1,int param_2,uint param_3,int param_4)

{
  return CONCAT44(param_4 - (param_2 + (uint)(param_3 < param_1)),param_3 - param_1);
}



/* ======================================================================
 * 0001b154  mul3
 * ====================================================================== */

longlong mul3(uint param_1,uint param_2)

{
  return (ulonglong)param_1 * (ulonglong)param_2;
}



/* ======================================================================
 * 0001b160  switch8_r3
 * ====================================================================== */

/* WARNING: This is an inlined function */

void switch8_r3(void)

{
  byte bVar1;
  uint in_r3;
  int in_lr;
  
  if (in_r3 < *(byte *)(in_lr + -1)) {
    bVar1 = *(byte *)(in_lr + in_r3);
  }
  else {
    bVar1 = *(byte *)(in_lr + (uint)*(byte *)(in_lr + -1));
  }
                    /* WARNING: Could not recover jumptable at 0x0001b174. Too many branches */
                    /* WARNING: Treating indirect jump as call */
  (*(code *)(in_lr + (uint)bVar1 * 2))();
  return;
}



/* ======================================================================
 * 0001b18c  __udivsi3
 * ====================================================================== */

/* WARNING: Removing unreachable block (ram,0x0001b0e0) */
/* WARNING: Removing unreachable block (ram,0x0001b0e4) */

undefined8 __udivsi3(uint param_1,uint param_2)

{
  int iVar1;
  undefined4 extraout_r1;
  undefined4 uVar2;
  uint uVar3;
  uint uVar4;
  int iVar5;
  undefined4 *unaff_r5;
  undefined4 *puVar6;
  undefined4 uVar7;
  int in_lr;
  bool bVar8;
  bool bVar9;
  bool bVar10;
  bool bVar11;
  bool bVar12;
  uint in_cpsr;
  undefined8 uVar13;
  int iStack00000010;
  undefined4 *puStack00000014;
  int iStack0000001c;
  undefined4 uStack00000020;
  undefined4 uStack00000024;
  undefined4 uStack_24;
  undefined4 uStack_20;
  undefined4 uStack_1c;
  int iStack_18;
  undefined4 uStack_14;
  
  iVar5 = 0;
  if (param_1 >> 4 < param_2) {
LAB_0001afe4:
    bVar8 = param_2 <= param_1 >> 3;
    if (bVar8) {
      param_1 = param_1 + param_2 * -8;
    }
    bVar9 = param_2 <= param_1 >> 2;
    if (bVar9) {
      param_1 = param_1 + param_2 * -4;
    }
    bVar10 = param_2 <= param_1 >> 1;
    if (bVar10) {
      param_1 = param_1 + param_2 * -2;
    }
    uVar3 = param_1 - param_2;
    if (param_2 > param_1) {
      uVar3 = param_1;
    }
    return CONCAT44(uVar3,(((iVar5 * 2 + (uint)bVar8) * 2 + (uint)bVar9) * 2 + (uint)bVar10) * 2 +
                          (uint)(param_2 <= param_1));
  }
  if (param_1 >> 8 < param_2) {
    bVar8 = param_2 <= param_1 >> 7;
    if (bVar8) {
      param_1 = param_1 + param_2 * -0x80;
    }
    bVar9 = param_2 <= param_1 >> 6;
    if (bVar9) {
      param_1 = param_1 + param_2 * -0x40;
    }
    bVar10 = param_2 <= param_1 >> 5;
    if (bVar10) {
      param_1 = param_1 + param_2 * -0x20;
    }
    bVar11 = param_2 <= param_1 >> 4;
    if (bVar11) {
      param_1 = param_1 + param_2 * -0x10;
    }
    iVar5 = (((uint)bVar8 * 2 + (uint)bVar9) * 2 + (uint)bVar10) * 2 + (uint)bVar11;
    goto LAB_0001afe4;
  }
  uVar3 = param_2 << 6;
  iStack_18 = -0x4000000;
  if (param_1 >> 8 < uVar3) goto LAB_0001b078;
  uVar3 = param_2 << 0xc;
  iStack_18 = -0x100000;
  if (param_1 >> 8 < uVar3) goto LAB_0001b078;
  uVar3 = param_2 << 0x12;
  iStack_18 = -0x4000;
  if (uVar3 <= param_1 >> 8) {
    iStack_18 = -0x100;
    uVar3 = param_2 << 0x18;
  }
  bVar8 = uVar3 == 0;
  iStack00000010 = -uVar3;
  if (!bVar8) {
    do {
      if (bVar8) {
        uVar3 = uVar3 >> 6;
      }
LAB_0001b078:
      bVar8 = uVar3 <= param_1 >> 7;
      if (bVar8) {
        param_1 = param_1 + uVar3 * -0x80;
      }
      bVar9 = uVar3 <= param_1 >> 6;
      if (bVar9) {
        param_1 = param_1 + uVar3 * -0x40;
      }
      bVar10 = uVar3 <= param_1 >> 5;
      if (bVar10) {
        param_1 = param_1 + uVar3 * -0x20;
      }
      bVar11 = uVar3 <= param_1 >> 4;
      if (bVar11) {
        param_1 = param_1 + uVar3 * -0x10;
      }
      bVar12 = uVar3 <= param_1 >> 3;
      if (bVar12) {
        param_1 = param_1 + uVar3 * -8;
      }
      uVar4 = ((((iStack_18 * 2 + (uint)bVar8) * 2 + (uint)bVar9) * 2 + (uint)bVar10) * 2 +
              (uint)bVar11) * 2 + (uint)bVar12;
      bVar9 = uVar3 <= param_1 >> 2;
      if (bVar9) {
        param_1 = param_1 + uVar3 * -4;
      }
      iStack_18 = uVar4 * 2 + (uint)bVar9;
      bVar8 = true;
    } while (CARRY4(uVar4,uVar4) || CARRY4(uVar4 * 2,(uint)bVar9));
    bVar8 = uVar3 <= param_1 >> 1;
    if (bVar8) {
      param_1 = param_1 + uVar3 * -2;
    }
    uVar4 = param_1 - uVar3;
    if (uVar3 > param_1) {
      uVar4 = param_1;
    }
    return CONCAT44(uVar4,(iStack_18 * 2 + (uint)bVar8) * 2 + (uint)(uVar3 <= param_1));
  }
  uStack_14 = 0;
  uStack_1c = 2;
  puStack00000014 = &uStack_20;
  uStack_20 = 2;
  iStack0000001c = in_lr + -4;
  bVar8 = (in_cpsr >> 0x1e & 1) != 0;
  uStack00000024 = 0;
  uStack00000020 = 0;
  puVar6 = &uStack_24;
  uStack_24 = 0;
  uVar7 = 0x16650;
  uVar13 = exc_build_indication_and_spin(puVar6);
  iVar5 = DAT_00016680;
  uVar2 = (undefined4)((ulonglong)uVar13 >> 0x20);
  iVar1 = (int)uVar13;
  if (bVar8) {
    uVar7 = *unaff_r5;
    puVar6 = (undefined4 *)unaff_r5[-1];
    iVar1 = unaff_r5[-5];
  }
  puVar6[-1] = uVar7;
  puVar6[-2] = 0;
  if (iVar1 == 0) {
    iVar1 = 0x10;
  }
  else {
    iVar1 = *(int *)(DAT_00016680 + 0x24) << 0x1f;
    if (iVar1 != 0) goto LAB_00016670;
    *(undefined4 *)(DAT_00016680 + 0x24) = 0x11;
    fw_delay_loop(0x28);
    iVar1 = 1;
    uVar2 = extraout_r1;
  }
  *(int *)(iVar5 + 0x24) = iVar1;
LAB_00016670:
  return CONCAT44(uVar2,iVar1);
}



/* ======================================================================
 * 0001b1a8  pas_rate_for_try_count
 * ====================================================================== */

void pas_rate_for_try_count(int param_1,char param_2,char *param_3)

{
  uint uVar1;
  byte bVar2;
  byte bVar3;
  
  bVar2 = param_2 + 1;
  uVar1 = 0x17;
  while( true ) {
    if ((uVar1 & 1) == 0) {
      bVar3 = *(byte *)(param_1 + 8 + (uVar1 >> 1)) & 0xf;
    }
    else {
      bVar3 = *(byte *)(param_1 + 8 + (uVar1 >> 1)) >> 4;
    }
    if (bVar2 < bVar3) break;
    bVar2 = bVar2 - bVar3;
    uVar1 = uVar1 - 1 & 0xff;
    if (uVar1 == 0xff) {
      return;
    }
  }
  *param_3 = bVar3 - bVar2;
  return;
}


