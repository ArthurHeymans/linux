// SPDX-License-Identifier: GPL-2.0-only
/*
 * mac80211 glue code for mac80211 ST-Ericsson CW1200 drivers
 * DebugFS code
 *
 * Copyright (c) 2010, ST-Ericsson
 * Author: Dmitry Tarnyagin <dmitry.tarnyagin@lockless.no>
 */

#include <linux/module.h>
#include <linux/capability.h>
#include <linux/debugfs.h>
#include <linux/seq_file.h>
#include "cw1200.h"
#include "bh.h"
#include "debug.h"
#include "fwio.h"
#include "hwio.h"
#include "wsm.h"

static bool unsafe_debugfs;
module_param(unsafe_debugfs, bool, 0600);
MODULE_PARM_DESC(unsafe_debugfs,
		 "Allow raw XR819 AHB/APB writes through debugfs");

/* join_status */
static const char * const cw1200_debug_join_status[] = {
	"passive",
	"monitor",
	"station (joining)",
	"station (not authenticated yet)",
	"station",
	"adhoc",
	"access point",
};

/* WSM_JOIN_PREAMBLE_... */
static const char * const cw1200_debug_preamble[] = {
	"long",
	"short",
	"long on 1 and 2 Mbps",
};


static const char * const cw1200_debug_link_id[] = {
	"OFF",
	"REQ",
	"SOFT",
	"HARD",
	"RESET",
	"RESET_REMAP",
};

static const char *cw1200_debug_mode(int mode)
{
	switch (mode) {
	case NL80211_IFTYPE_UNSPECIFIED:
		return "unspecified";
	case NL80211_IFTYPE_MONITOR:
		return "monitor";
	case NL80211_IFTYPE_STATION:
		return "station";
	case NL80211_IFTYPE_ADHOC:
		return "adhoc";
	case NL80211_IFTYPE_MESH_POINT:
		return "mesh point";
	case NL80211_IFTYPE_AP:
		return "access point";
	case NL80211_IFTYPE_P2P_CLIENT:
		return "p2p client";
	case NL80211_IFTYPE_P2P_GO:
		return "p2p go";
	default:
		return "unsupported";
	}
}

static void cw1200_queue_status_show(struct seq_file *seq,
				     struct cw1200_queue *q)
{
	int i;
	seq_printf(seq, "Queue       %d:\n", q->queue_id);
	seq_printf(seq, "  capacity: %zu\n", q->capacity);
	seq_printf(seq, "  queued:   %zu\n", q->num_queued);
	seq_printf(seq, "  pending:  %zu\n", q->num_pending);
	seq_printf(seq, "  sent:     %zu\n", q->num_sent);
	seq_printf(seq, "  locked:   %s\n", q->tx_locked_cnt ? "yes" : "no");
	seq_printf(seq, "  overfull: %s\n", q->overfull ? "yes" : "no");
	seq_puts(seq,   "  link map: 0-> ");
	for (i = 0; i < q->stats->map_capacity; ++i)
		seq_printf(seq, "%.2d ", q->link_map_cache[i]);
	seq_printf(seq, "<-%zu\n", q->stats->map_capacity);
}

static void cw1200_debug_print_map(struct seq_file *seq,
				   struct cw1200_common *priv,
				   const char *label,
				   u32 map)
{
	int i;
	seq_printf(seq, "%s0-> ", label);
	for (i = 0; i < priv->tx_queue_stats.map_capacity; ++i)
		seq_printf(seq, "%s ", (map & BIT(i)) ? "**" : "..");
	seq_printf(seq, "<-%zu\n", priv->tx_queue_stats.map_capacity - 1);
}

static int cw1200_status_show(struct seq_file *seq, void *v)
{
	int i;
	struct list_head *item;
	struct cw1200_common *priv = seq->private;
	struct cw1200_debug_priv *d = priv->debug;

	seq_puts(seq,   "CW1200 Wireless LAN driver status\n");
	seq_printf(seq, "Hardware:   %d.%d\n",
		   priv->wsm_caps.hw_id,
		   priv->wsm_caps.hw_subid);
	seq_printf(seq, "Firmware:   %s %d.%d\n",
		   cw1200_fw_types[priv->wsm_caps.fw_type],
		   priv->wsm_caps.fw_ver,
		   priv->wsm_caps.fw_build);
	seq_printf(seq, "FW API:     %d\n",
		   priv->wsm_caps.fw_api);
	seq_printf(seq, "FW caps:    0x%.4X\n",
		   priv->wsm_caps.fw_cap);
	seq_printf(seq, "FW label:  '%s'\n",
		   priv->wsm_caps.fw_label);
	seq_printf(seq, "Mode:       %s%s\n",
		   cw1200_debug_mode(priv->mode),
		   priv->listening ? " (listening)" : "");
	seq_printf(seq, "Join state: %s\n",
		   cw1200_debug_join_status[priv->join_status]);
	if (priv->channel)
		seq_printf(seq, "Channel:    %d%s\n",
			   priv->channel->hw_value,
			   priv->channel_switch_in_progress ?
			   " (switching)" : "");
	if (priv->rx_filter.promiscuous)
		seq_puts(seq,   "Filter:     promisc\n");
	else if (priv->rx_filter.fcs)
		seq_puts(seq,   "Filter:     fcs\n");
	if (priv->rx_filter.bssid)
		seq_puts(seq,   "Filter:     bssid\n");
	if (!priv->disable_beacon_filter)
		seq_puts(seq,   "Filter:     beacons\n");

	if (priv->enable_beacon ||
	    priv->mode == NL80211_IFTYPE_AP ||
	    priv->mode == NL80211_IFTYPE_ADHOC ||
	    priv->mode == NL80211_IFTYPE_MESH_POINT ||
	    priv->mode == NL80211_IFTYPE_P2P_GO)
		seq_printf(seq, "Beaconing:  %s\n",
			   priv->enable_beacon ?
			   "enabled" : "disabled");

	for (i = 0; i < 4; ++i)
		seq_printf(seq, "EDCA(%d):    %d, %d, %d, %d, %d\n", i,
			   priv->edca.params[i].cwmin,
			   priv->edca.params[i].cwmax,
			   priv->edca.params[i].aifns,
			   priv->edca.params[i].txop_limit,
			   priv->edca.params[i].max_rx_lifetime);

	if (priv->join_status == CW1200_JOIN_STATUS_STA) {
		static const char *pm_mode = "unknown";
		switch (priv->powersave_mode.mode) {
		case WSM_PSM_ACTIVE:
			pm_mode = "off";
			break;
		case WSM_PSM_PS:
			pm_mode = "on";
			break;
		case WSM_PSM_FAST_PS:
			pm_mode = "dynamic";
			break;
		}
		seq_printf(seq, "Preamble:   %s\n",
			   cw1200_debug_preamble[priv->association_mode.preamble]);
		seq_printf(seq, "AMPDU spcn: %d\n",
			   priv->association_mode.mpdu_start_spacing);
		seq_printf(seq, "Basic rate: 0x%.8X\n",
			   le32_to_cpu(priv->association_mode.basic_rate_set));
		seq_printf(seq, "Bss lost:   %d beacons\n",
			   priv->bss_params.beacon_lost_count);
		seq_printf(seq, "AID:        %d\n",
			   priv->bss_params.aid);
		seq_printf(seq, "Rates:      0x%.8X\n",
			   priv->bss_params.operational_rate_set);
		seq_printf(seq, "Powersave:  %s\n", pm_mode);
	}
	seq_printf(seq, "HT:         %s\n",
		   cw1200_is_ht(&priv->ht_info) ? "on" : "off");
	if (cw1200_is_ht(&priv->ht_info)) {
		seq_printf(seq, "Greenfield: %s\n",
			   cw1200_ht_greenfield(&priv->ht_info) ? "yes" : "no");
		seq_printf(seq, "AMPDU dens: %d\n",
			   cw1200_ht_ampdu_density(&priv->ht_info));
	}
	seq_printf(seq, "RSSI thold: %d\n",
		   priv->cqm_rssi_thold);
	seq_printf(seq, "RSSI hyst:  %d\n",
		   priv->cqm_rssi_hyst);
	seq_printf(seq, "Long retr:  %d\n",
		   priv->long_frame_max_tx_count);
	seq_printf(seq, "Short retr: %d\n",
		   priv->short_frame_max_tx_count);
	spin_lock_bh(&priv->tx_policy_cache.lock);
	i = 0;
	list_for_each(item, &priv->tx_policy_cache.used)
		++i;
	spin_unlock_bh(&priv->tx_policy_cache.lock);
	seq_printf(seq, "RC in use:  %d\n", i);

	seq_puts(seq, "\n");
	for (i = 0; i < 4; ++i) {
		cw1200_queue_status_show(seq, &priv->tx_queue[i]);
		seq_puts(seq, "\n");
	}

	cw1200_debug_print_map(seq, priv, "Link map:   ",
			       priv->link_id_map);
	cw1200_debug_print_map(seq, priv, "Asleep map: ",
			       priv->sta_asleep_mask);
	cw1200_debug_print_map(seq, priv, "PSPOLL map: ",
			       priv->pspoll_mask);

	seq_puts(seq, "\n");

	for (i = 0; i < CW1200_MAX_STA_IN_AP_MODE; ++i) {
		if (priv->link_id_db[i].status) {
			seq_printf(seq, "Link %d:     %s, %pM\n",
				   i + 1,
				   cw1200_debug_link_id[priv->link_id_db[i].status],
				   priv->link_id_db[i].mac);
		}
	}

	seq_puts(seq, "\n");

	seq_printf(seq, "BH status:  %s\n",
		   atomic_read(&priv->bh_term) ? "terminated" : "alive");
	seq_printf(seq, "Pending RX: %d\n",
		   atomic_read(&priv->bh_rx));
	seq_printf(seq, "Pending TX: %d\n",
		   atomic_read(&priv->bh_tx));
	if (priv->bh_error)
		seq_printf(seq, "BH errcode: %d\n",
			   priv->bh_error);
	seq_printf(seq, "TX bufs:    %d x %d bytes\n",
		   priv->wsm_caps.input_buffers,
		   priv->wsm_caps.input_buffer_size);
	seq_printf(seq, "Used bufs:  %d\n",
		   priv->hw_bufs_used);
	seq_printf(seq, "Powermgmt:  %s\n",
		   priv->powersave_enabled ? "on" : "off");
	seq_printf(seq, "Device:     %s\n",
		   priv->device_can_sleep ? "asleep" : "awake");

	spin_lock(&priv->wsm_cmd.lock);
	seq_printf(seq, "WSM status: %s\n",
		   priv->wsm_cmd.done ? "idle" : "active");
	seq_printf(seq, "WSM cmd:    0x%.4X (%td bytes)\n",
		   priv->wsm_cmd.cmd, priv->wsm_cmd.len);
	seq_printf(seq, "WSM retval: %d\n",
		   priv->wsm_cmd.ret);
	spin_unlock(&priv->wsm_cmd.lock);

	seq_printf(seq, "Datapath:   %s\n",
		   atomic_read(&priv->tx_lock) ? "locked" : "unlocked");
	if (atomic_read(&priv->tx_lock))
		seq_printf(seq, "TXlock cnt: %d\n",
			   atomic_read(&priv->tx_lock));

	seq_printf(seq, "TXed:       %d\n",
		   d->tx);
	seq_printf(seq, "AGG TXed:   %d\n",
		   d->tx_agg);
	seq_printf(seq, "MULTI TXed: %d (%d)\n",
		   d->tx_multi, d->tx_multi_frames);
	seq_printf(seq, "RXed:       %d\n",
		   d->rx);
	seq_printf(seq, "AGG RXed:   %d\n",
		   d->rx_agg);
	seq_printf(seq, "TX miss:    %d\n",
		   d->tx_cache_miss);
	seq_printf(seq, "TX align:   %d\n",
		   d->tx_align);
	seq_printf(seq, "TX burst:   %d\n",
		   d->tx_burst);
	seq_printf(seq, "TX TTL:     %d\n",
		   d->tx_ttl);
	seq_printf(seq, "AGG meta:   %d (%d heads, %d members without)\n",
		   d->tx_agg_metadata, d->tx_agg_metadata_head,
		   d->tx_agg_without_metadata);
	seq_printf(seq, "AGG report: %d ctl, %d len, %d ack, %d invalid\n",
		   d->tx_agg_metadata_ctl, d->tx_agg_metadata_len,
		   d->tx_agg_metadata_ack, d->tx_agg_metadata_invalid);
	seq_printf(seq, "AGG BAR req: %d\n", d->tx_ampdu_no_back);
	seq_printf(seq, "TX confirm: %d ok, %d fail\n",
		   d->tx_confirm_ok, d->tx_confirm_fail);
	seq_printf(seq, "Conf unmatched: %d (%d failed)\n",
		   d->tx_confirm_unmatched, d->tx_confirm_unmatched_fail);
	seq_printf(seq, "Scan:       %s\n",
		   atomic_read(&priv->scan.in_progress) ? "active" : "idle");

	return 0;
}

DEFINE_SHOW_ATTRIBUTE(cw1200_status);

static int cw1200_counters_show(struct seq_file *seq, void *v)
{
	int ret;
	struct cw1200_common *priv = seq->private;
	struct wsm_mib_counters_table counters;

	ret = wsm_get_counters_table(priv, &counters);
	if (ret)
		return ret;

#define PUT_COUNTER(tab, name) \
	seq_printf(seq, "%s:" tab "%d\n", #name, \
		__le32_to_cpu(counters.name))

	PUT_COUNTER("\t\t", plcp_errors);
	PUT_COUNTER("\t\t", fcs_errors);
	PUT_COUNTER("\t\t", tx_packets);
	PUT_COUNTER("\t\t", rx_packets);
	PUT_COUNTER("\t\t", rx_packet_errors);
	PUT_COUNTER("\t",   rx_decryption_failures);
	PUT_COUNTER("\t\t", rx_mic_failures);
	PUT_COUNTER("\t",   rx_no_key_failures);
	PUT_COUNTER("\t",   tx_multicast_frames);
	PUT_COUNTER("\t",   tx_frames_success);
	PUT_COUNTER("\t",   tx_frame_failures);
	PUT_COUNTER("\t",   tx_frames_retried);
	PUT_COUNTER("\t",   tx_frames_multi_retried);
	PUT_COUNTER("\t",   rx_frame_duplicates);
	PUT_COUNTER("\t\t", rts_success);
	PUT_COUNTER("\t\t", rts_failures);
	PUT_COUNTER("\t\t", ack_failures);
	PUT_COUNTER("\t",   rx_multicast_frames);
	PUT_COUNTER("\t",   rx_frames_success);
	PUT_COUNTER("\t",   rx_cmac_icv_errors);
	PUT_COUNTER("\t\t", rx_cmac_replays);
	PUT_COUNTER("\t",   rx_mgmt_ccmp_replays);

#undef PUT_COUNTER

	return 0;
}

DEFINE_SHOW_ATTRIBUTE(cw1200_counters);

static const char *cw1200_bh_rx_diag_reason(u16 reason)
{
	switch (reason) {
	case CW1200_BH_RX_DIAG_MESSAGE:
		return "message";
	case CW1200_BH_RX_DIAG_INVALID_CTRL_LENGTH:
		return "invalid-ctrl-length";
	case CW1200_BH_RX_DIAG_DATA_READ_FAILED:
		return "data-read-failed";
	case CW1200_BH_RX_DIAG_INVALID_WSM_LENGTH:
		return "invalid-wsm-length";
	case CW1200_BH_RX_DIAG_SEQUENCE_MISMATCH:
		return "sequence-mismatch";
	case CW1200_BH_RX_DIAG_EXCEPTION:
		return "exception";
	case CW1200_BH_RX_DIAG_CREDIT_FAILED:
		return "credit-failed";
	case CW1200_BH_RX_DIAG_HANDLER_FAILED:
		return "handler-failed";
	default:
		return "unknown";
	}
}

static int cw1200_bh_rx_trace_show(struct seq_file *seq, void *v)
{
	struct cw1200_common *priv = seq->private;
	struct cw1200_bh_rx_diag *diag = &priv->bh_rx_diag;
	struct cw1200_bh_rx_diag_entry entry;
	unsigned long flags;
	u32 count, head, available, first, i, j;

	spin_lock_irqsave(&diag->lock, flags);
	count = diag->count;
	head = diag->head;
	spin_unlock_irqrestore(&diag->lock, flags);

	available = min_t(u32, count, CW1200_BH_RX_DIAG_DEPTH);
	first = (head + CW1200_BH_RX_DIAG_DEPTH - available) %
		CW1200_BH_RX_DIAG_DEPTH;
	seq_printf(seq, "count=%u available=%u head=%u\n", count, available,
		   head);
	seq_puts(seq,
		 "ordinal timestamp_ns reason ctrl read alloc wsm id seq expected cmd result data\n");

	for (i = 0; i < available; i++) {
		u32 slot = (first + i) % CW1200_BH_RX_DIAG_DEPTH;

		spin_lock_irqsave(&diag->lock, flags);
		entry = diag->entries[slot];
		spin_unlock_irqrestore(&diag->lock, flags);

		seq_printf(seq,
			   "%u %llu %s %04x->%04x %u %u %u %04x %u %u %04x %d ",
			   entry.ordinal, entry.timestamp_ns,
			   cw1200_bh_rx_diag_reason(entry.reason),
			   entry.ctrl_before, entry.ctrl_after,
			   entry.read_len, entry.alloc_len, entry.wsm_len,
			   entry.wsm_id, entry.wsm_seq, entry.expected_seq,
			   entry.expected_cmd, entry.result);
		for (j = 0; j < entry.data_len; j++)
			seq_printf(seq, "%02x", entry.data[j]);
		seq_putc(seq, '\n');
	}

	return 0;
}

DEFINE_SHOW_ATTRIBUTE(cw1200_bh_rx_trace);

static ssize_t cw1200_wsm_dumps(struct file *file,
	const char __user *user_buf, size_t count, loff_t *ppos)
{
	struct cw1200_common *priv = file->private_data;
	char buf[1];

	if (!count)
		return -EINVAL;
	if (copy_from_user(buf, user_buf, 1))
		return -EFAULT;

	if (buf[0] == '1')
		priv->wsm_enable_wsm_dumps = 1;
	else
		priv->wsm_enable_wsm_dumps = 0;

	return count;
}

static const struct file_operations fops_wsm_dumps = {
	.open = simple_open,
	.write = cw1200_wsm_dumps,
	.llseek = default_llseek,
};

static int cw1200_debug_mem_show(struct seq_file *seq, void *v)
{
	struct cw1200_debug_mem *mem = seq->private;
	u32 value;
	int ret;

	guard(mutex)(&mem->lock);
	if (mem->priv->debug->state == CW1200_DEBUG_NORMAL)
		return -EBUSY;
	if (mem->ahb)
		ret = cw1200_ahb_read_32(mem->priv, mem->address, &value);
	else
		ret = cw1200_apb_read_32(mem->priv, mem->address, &value);
	if (ret)
		return ret;

	seq_printf(seq, "0x%08x: 0x%08x\n", mem->address, value);
	return 0;
}

static int cw1200_debug_mem_open(struct inode *inode, struct file *file)
{
	return single_open(file, cw1200_debug_mem_show, inode->i_private);
}

static ssize_t cw1200_debug_mem_write(struct file *file,
				      const char __user *user_buf,
				      size_t count, loff_t *ppos)
{
	struct seq_file *seq = file->private_data;
	struct cw1200_debug_mem *mem = seq->private;
	char buf[64];
	u32 address;
	u32 value;
	int fields;
	int ret;

	if (!count || count >= sizeof(buf))
		return -EINVAL;
	if (copy_from_user(buf, user_buf, count))
		return -EFAULT;
	buf[count] = '\0';

	fields = sscanf(buf, "%x %x", &address, &value);
	if (fields < 1)
		return -EINVAL;
	if (!IS_ALIGNED(address, sizeof(value)))
		return -EINVAL;

	guard(mutex)(&mem->lock);
	mem->address = address;
	if (fields == 1)
		return count;
	if (mem->priv->debug->state == CW1200_DEBUG_NORMAL)
		return -EBUSY;
	if (!unsafe_debugfs || !capable(CAP_SYS_RAWIO))
		return -EPERM;
	if (mem->ahb)
		ret = cw1200_ahb_write_32(mem->priv, address, value);
	else
		ret = cw1200_apb_write_32(mem->priv, address, value);

	return ret ? ret : count;
}

static const struct file_operations cw1200_debug_mem_fops = {
	.owner = THIS_MODULE,
	.open = cw1200_debug_mem_open,
	.read = seq_read,
	.write = cw1200_debug_mem_write,
	.llseek = seq_lseek,
	.release = single_release,
};

static int cw1200_debug_halt_show(struct seq_file *seq, void *v)
{
	struct cw1200_common *priv = seq->private;

	seq_printf(seq, "%u\n", priv->debug->state);
	return 0;
}

static int cw1200_debug_halt_open(struct inode *inode, struct file *file)
{
	return single_open(file, cw1200_debug_halt_show, inode->i_private);
}

static ssize_t cw1200_debug_halt_write(struct file *file,
				       const char __user *user_buf,
				       size_t count, loff_t *ppos)
{
	struct seq_file *seq = file->private_data;
	struct cw1200_common *priv = seq->private;
	struct cw1200_debug_priv *debug = priv->debug;
	bool halt;
	u32 config;
	int ret;

	if (!unsafe_debugfs || !capable(CAP_SYS_RAWIO))
		return -EPERM;
	ret = kstrtobool_from_user(user_buf, count, &halt);
	if (ret)
		return ret;

	guard(mutex)(&debug->control_lock);
	if (halt && debug->state == CW1200_DEBUG_HALTED)
		return count;
	if (!halt && debug->state == CW1200_DEBUG_NORMAL)
		return count;

	if (halt && debug->state == CW1200_DEBUG_PAYLOAD) {
		ret = cw1200_reg_read_32(priv, ST90TDS_CONFIG_REG_ID, &config);
		if (ret)
			return ret;
		ret = cw1200_reg_write_32(priv, ST90TDS_CONFIG_REG_ID,
					  config |
					  ST90TDS_CONFIG_CPU_RESET_BIT |
					  ST90TDS_CONFIG_ACCESS_MODE_BIT);
		if (!ret)
			debug->state = CW1200_DEBUG_HALTED;
		return ret ? ret : count;
	}

	if (halt) {
		wsm_lock_tx(priv);
		ret = cw1200_bh_suspend(priv);
		if (ret)
			goto unlock_tx;
		ret = cw1200_reg_read_32(priv, ST90TDS_CONFIG_REG_ID,
					 &debug->saved_config);
		if (ret)
			goto resume_bh;
		ret = cw1200_reg_write_32(priv, ST90TDS_CONFIG_REG_ID,
					  debug->saved_config |
					  ST90TDS_CONFIG_CPU_RESET_BIT |
					  ST90TDS_CONFIG_ACCESS_MODE_BIT);
		if (ret)
			goto resume_bh;
		debug->state = CW1200_DEBUG_HALTED;
		return count;
	}

	ret = cw1200_reg_write_32(priv, ST90TDS_CONFIG_REG_ID,
				  debug->saved_config);
	if (ret)
		return ret;
	debug->state = CW1200_DEBUG_NORMAL;
	ret = cw1200_bh_resume(priv);
	wsm_unlock_tx(priv);
	return ret ? ret : count;

resume_bh:
	cw1200_bh_resume(priv);
unlock_tx:
	wsm_unlock_tx(priv);
	return ret;
}

static const struct file_operations cw1200_debug_halt_fops = {
	.owner = THIS_MODULE,
	.open = cw1200_debug_halt_open,
	.read = seq_read,
	.write = cw1200_debug_halt_write,
	.llseek = seq_lseek,
	.release = single_release,
};

static ssize_t cw1200_debug_upload_write(struct file *file,
					 const char __user *user_buf,
					 size_t count, loff_t *ppos)
{
	struct cw1200_common *priv = file->private_data;
	struct cw1200_debug_priv *debug = priv->debug;
	void *buf;
	int ret;

	if (!unsafe_debugfs || !capable(CAP_SYS_RAWIO))
		return -EPERM;
	if (debug->state != CW1200_DEBUG_HALTED)
		return -EBUSY;
	if (!count || !IS_ALIGNED(*ppos, sizeof(u32)) ||
	    !IS_ALIGNED(count, sizeof(u32)) || *ppos + count > SZ_64K)
		return -EINVAL;

	buf = memdup_user(user_buf, count);
	if (IS_ERR(buf))
		return PTR_ERR(buf);
	ret = cw1200_ahb_write(priv, AHB_MEMORY_ADDRESS + *ppos, buf, count);
	kfree(buf);
	if (ret)
		return ret;
	*ppos += count;
	return count;
}

static const struct file_operations cw1200_debug_upload_fops = {
	.owner = THIS_MODULE,
	.open = simple_open,
	.write = cw1200_debug_upload_write,
	.llseek = default_llseek,
};

static ssize_t cw1200_debug_run_write(struct file *file,
				      const char __user *user_buf,
				      size_t count, loff_t *ppos)
{
	struct cw1200_common *priv = file->private_data;
	struct cw1200_debug_priv *debug = priv->debug;
	bool run;
	u32 config;
	int ret;

	if (!unsafe_debugfs || !capable(CAP_SYS_RAWIO))
		return -EPERM;
	ret = kstrtobool_from_user(user_buf, count, &run);
	if (ret)
		return ret;
	if (!run)
		return -EINVAL;

	guard(mutex)(&debug->control_lock);
	if (debug->state != CW1200_DEBUG_HALTED)
		return -EBUSY;
	config = (debug->saved_config | ST90TDS_CONFIG_ACCESS_MODE_BIT) &
		 ~(ST90TDS_CONFIG_CPU_RESET_BIT | ST90TDS_CONFIG_CPU_CLK_DIS_BIT);
	ret = cw1200_reg_write_32(priv, ST90TDS_CONFIG_REG_ID, config);
	if (ret)
		return ret;
	debug->state = CW1200_DEBUG_PAYLOAD;
	return count;
}

static const struct file_operations cw1200_debug_run_fops = {
	.owner = THIS_MODULE,
	.open = simple_open,
	.write = cw1200_debug_run_write,
	.llseek = default_llseek,
};

#define XR819_TRACE_ADDRESS 0x0900fd20
#define XR819_TRACE_WORDS 12
#define XR819_TRACE_MAGIC 0x54584558

static int cw1200_debug_fw_trace_show(struct seq_file *seq, void *v)
{
	struct cw1200_common *priv = seq->private;
	struct cw1200_debug_priv *debug = priv->debug;
	__le32 raw[XR819_TRACE_WORDS];
	u32 config;
	u32 words[XR819_TRACE_WORDS];
	int ret;
	int i;

	guard(mutex)(&debug->control_lock);

	/* A live BH owns the device access path. After a fatal BH exit, stop the
	 * firmware core in place so packet SRAM can be read before any reload
	 * destroys the failure record. */
	if (debug->state == CW1200_DEBUG_NORMAL) {
		if (!READ_ONCE(priv->bh_error))
			return -EBUSY;
		ret = cw1200_reg_read_32(priv, ST90TDS_CONFIG_REG_ID, &config);
		if (ret)
			return ret;
		debug->saved_config = config;
		config |= ST90TDS_CONFIG_CPU_RESET_BIT |
			  ST90TDS_CONFIG_ACCESS_MODE_BIT;
		config &= ~(ST90TDS_CONFIG_AHB_PRFETCH_BIT |
			    ST90TDS_CONFIG_PRFETCH_BIT);
		ret = cw1200_reg_write_32(priv, ST90TDS_CONFIG_REG_ID, config);
		if (ret)
			return ret;
		debug->state = CW1200_DEBUG_HALTED;
		msleep(30);
	}

	ret = cw1200_ahb_read(priv, XR819_TRACE_ADDRESS, raw, sizeof(raw));
	if (ret)
		return ret;
	for (i = 0; i < XR819_TRACE_WORDS; i++)
		words[i] = le32_to_cpu(raw[i]);

	seq_printf(seq, "magic: 0x%08x%s\n", words[0],
		   words[0] == XR819_TRACE_MAGIC ? " (TXEX)" : " (invalid)");
	seq_printf(seq, "version: %u\n", words[1]);
	seq_printf(seq, "stages: 0x%08x\n", words[2]);
	seq_printf(seq, "  published: %u\n", !!(words[2] & BIT(0)));
	seq_printf(seq, "  go:        %u\n", !!(words[2] & BIT(1)));
	seq_printf(seq, "  fiq:       %u\n", !!(words[2] & BIT(2)));
	seq_printf(seq, "  fifo_pop:  %u\n", !!(words[2] & BIT(3)));
	seq_printf(seq, "  bit23:     %u\n", !!(words[2] & BIT(4)));
	seq_printf(seq, "  phase2:    %u\n", !!(words[2] & BIT(5)));
	seq_printf(seq, "  tx_start:  %u\n", !!(words[2] & BIT(6)));
	seq_printf(seq, "  phy_cmd2:  %u\n", !!(words[2] & BIT(7)));
	seq_printf(seq, "  success:   %u\n", !!(words[2] & BIT(8)));
	seq_printf(seq, "first_event: 0x%08x\n", words[3]);
	seq_printf(seq, "pipe_slot:   0x%08x\n", words[4]);
	seq_printf(seq, "fiq_pending: 0x%08x\n", words[5]);
	seq_printf(seq, "scheduler:   0x%08x\n", words[6]);
	seq_printf(seq, "phase2_evt:  0x%08x\n", words[7]);
	seq_printf(seq, "bit23_evt:   0x%08x\n", words[8]);
	seq_printf(seq, "pipe:        0x%08x\n", words[9]);
	seq_printf(seq, "phy_arg:     0x%08x\n", words[10]);
	seq_printf(seq, "success_arg: 0x%08x\n", words[11]);
	return 0;
}

static int cw1200_debug_fw_trace_open(struct inode *inode, struct file *file)
{
	return single_open(file, cw1200_debug_fw_trace_show, inode->i_private);
}

static const struct file_operations cw1200_debug_fw_trace_fops = {
	.owner = THIS_MODULE,
	.open = cw1200_debug_fw_trace_open,
	.read = seq_read,
	.llseek = seq_lseek,
	.release = single_release,
};

int cw1200_debug_init(struct cw1200_common *priv)
{
	int ret = -ENOMEM;
	struct cw1200_debug_priv *d = kzalloc_obj(struct cw1200_debug_priv);
	priv->debug = d;
	if (!d)
		return ret;

	mutex_init(&d->control_lock);
	d->ahb.priv = priv;
	d->ahb.ahb = true;
	d->ahb.address = AHB_MEMORY_ADDRESS;
	mutex_init(&d->ahb.lock);
	d->apb.priv = priv;
	d->apb.address = PAC_SHARED_MEMORY_SILICON;
	mutex_init(&d->apb.lock);

	d->debugfs_phy = debugfs_create_dir("cw1200",
					    priv->hw->wiphy->debugfsdir);
	debugfs_create_file("status", 0400, d->debugfs_phy, priv,
			    &cw1200_status_fops);
	debugfs_create_file("counters", 0400, d->debugfs_phy, priv,
			    &cw1200_counters_fops);
	debugfs_create_file("bh_rx_trace", 0400, d->debugfs_phy, priv,
			    &cw1200_bh_rx_trace_fops);
	debugfs_create_file("wsm_dumps", 0200, d->debugfs_phy, priv,
			    &fops_wsm_dumps);
	if (priv->is_xr819) {
		debugfs_create_file("halt", 0600, d->debugfs_phy, priv,
				    &cw1200_debug_halt_fops);
		debugfs_create_file("upload", 0200, d->debugfs_phy, priv,
				    &cw1200_debug_upload_fops);
		debugfs_create_file("run", 0200, d->debugfs_phy, priv,
				    &cw1200_debug_run_fops);
		debugfs_create_file("ahb", 0600, d->debugfs_phy, &d->ahb,
				    &cw1200_debug_mem_fops);
		debugfs_create_file("apb", 0600, d->debugfs_phy, &d->apb,
				    &cw1200_debug_mem_fops);
		debugfs_create_file("fw_trace", S_IRUSR, d->debugfs_phy, priv,
				    &cw1200_debug_fw_trace_fops);
	}

	return 0;
}

void cw1200_debug_release(struct cw1200_common *priv)
{
	struct cw1200_debug_priv *d = priv->debug;
	if (d) {
		debugfs_remove_recursive(d->debugfs_phy);
		priv->debug = NULL;
		kfree(d);
	}
}
