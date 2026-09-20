/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * DebugFS code for ST-Ericsson CW1200 mac80211 driver
 *
 * Copyright (c) 2011, ST-Ericsson
 * Author: Dmitry Tarnyagin <dmitry.tarnyagin@lockless.no>
 */

#ifndef CW1200_DEBUG_H_INCLUDED
#define CW1200_DEBUG_H_INCLUDED

struct cw1200_debug_mem {
	struct cw1200_common *priv;
	struct mutex lock;
	u32 address;
	bool ahb;
};

enum cw1200_debug_state {
	CW1200_DEBUG_NORMAL,
	CW1200_DEBUG_HALTED,
	CW1200_DEBUG_PAYLOAD,
};

struct cw1200_debug_priv {
	struct dentry *debugfs_phy;
	struct cw1200_debug_mem ahb;
	struct cw1200_debug_mem apb;
	struct mutex control_lock;
	u32 saved_config;
	enum cw1200_debug_state state;
	int tx;
	int tx_agg;
	int rx;
	int rx_agg;
	int tx_multi;
	int tx_multi_frames;
	int tx_cache_miss;
	int tx_align;
	int tx_ttl;
	int tx_burst;
	int ba_cnt;
	int ba_acc;
	int ba_cnt_rx;
	int ba_acc_rx;
	int tx_agg_metadata;
	int tx_agg_metadata_head;
	int tx_agg_metadata_ctl;
	int tx_agg_metadata_len;
	int tx_agg_metadata_ack;
	int tx_agg_metadata_invalid;
	int tx_agg_without_metadata;
	int tx_ampdu_no_back;
	int tx_confirm_ok;
	int tx_confirm_fail;
	int tx_confirm_unmatched;
	int tx_confirm_unmatched_fail;
};

int cw1200_debug_init(struct cw1200_common *priv);
void cw1200_debug_release(struct cw1200_common *priv);

static inline void cw1200_debug_txed(struct cw1200_common *priv)
{
	++priv->debug->tx;
}

static inline void cw1200_debug_txed_agg(struct cw1200_common *priv)
{
	++priv->debug->tx_agg;
}

static inline void cw1200_debug_txed_agg_metadata(
	struct cw1200_common *priv, bool head, bool tx_ctl_ampdu,
	int len, int ack_len)
{
	++priv->debug->tx_agg_metadata;
	if (!head)
		return;
	++priv->debug->tx_agg_metadata_head;
	if (tx_ctl_ampdu)
		++priv->debug->tx_agg_metadata_ctl;
	priv->debug->tx_agg_metadata_len += len;
	priv->debug->tx_agg_metadata_ack += ack_len;
	if (!len || ack_len > len)
		++priv->debug->tx_agg_metadata_invalid;
}

static inline void cw1200_debug_txed_agg_without_metadata(
	struct cw1200_common *priv)
{
	++priv->debug->tx_agg_without_metadata;
}

static inline void cw1200_debug_ampdu_no_back(struct cw1200_common *priv)
{
	++priv->debug->tx_ampdu_no_back;
}

static inline void cw1200_debug_tx_confirm(struct cw1200_common *priv, bool failed)
{
	if (failed)
		++priv->debug->tx_confirm_fail;
	else
		++priv->debug->tx_confirm_ok;
}

/* A confirmation whose packet id no longer matches a queued item is dropped
 * silently by cw1200_tx_confirm_cb, so it never reaches mac80211. */
static inline void cw1200_debug_tx_confirm_unmatched(struct cw1200_common *priv,
						     bool failed)
{
	++priv->debug->tx_confirm_unmatched;
	if (failed)
		++priv->debug->tx_confirm_unmatched_fail;
}

static inline void cw1200_debug_txed_multi(struct cw1200_common *priv,
					   int count)
{
	++priv->debug->tx_multi;
	priv->debug->tx_multi_frames += count;
}

static inline void cw1200_debug_rxed(struct cw1200_common *priv)
{
	++priv->debug->rx;
}

static inline void cw1200_debug_rxed_agg(struct cw1200_common *priv)
{
	++priv->debug->rx_agg;
}

static inline void cw1200_debug_tx_cache_miss(struct cw1200_common *priv)
{
	++priv->debug->tx_cache_miss;
}

static inline void cw1200_debug_tx_align(struct cw1200_common *priv)
{
	++priv->debug->tx_align;
}

static inline void cw1200_debug_tx_ttl(struct cw1200_common *priv)
{
	++priv->debug->tx_ttl;
}

static inline void cw1200_debug_tx_burst(struct cw1200_common *priv)
{
	++priv->debug->tx_burst;
}

static inline void cw1200_debug_ba(struct cw1200_common *priv,
				   int ba_cnt, int ba_acc,
				   int ba_cnt_rx, int ba_acc_rx)
{
	priv->debug->ba_cnt = ba_cnt;
	priv->debug->ba_acc = ba_acc;
	priv->debug->ba_cnt_rx = ba_cnt_rx;
	priv->debug->ba_acc_rx = ba_acc_rx;
}

#endif /* CW1200_DEBUG_H_INCLUDED */
