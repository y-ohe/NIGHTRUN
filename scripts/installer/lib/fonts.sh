# shellcheck shell=bash disable=SC2034
# (SC2034: globals here are set for sibling modules sourced by install.sh)

nr_offer_pillow() {
    local manager="" package="" command=()
    if command -v apt-get >/dev/null; then
        manager="apt"
        package="python3-pil"
        command=(sudo apt-get install -y python3-pil)
    elif command -v dnf >/dev/null; then
        manager="dnf"
        package="python3-pillow"
        command=(sudo dnf install -y python3-pillow)
    elif command -v pacman >/dev/null; then
        manager="pacman"
        package="python-pillow"
        command=(sudo pacman -S --noconfirm python-pillow)
    fi

    nr_error "Pillow is required to rasterize Noto Sans JP into PSF2 fonts."
    if [[ -z "$manager" ]]; then
        nr_note "Install Python 3 Pillow with FreeType support, then rerun install.sh."
        return 1
    fi
    nr_note "The exact command that would run: ${command[*]}"
    nr_ask "Install $package via $manager? [1] Yes  [2] No (default)"
    [[ "$REPLY" == "1" ]] || return 1
    "${command[@]}" || return 1
    python3 -c 'from PIL import ImageFont; assert hasattr(ImageFont.FreeTypeFont, "get_variation_axes")' \
        || { nr_error "Pillow lacks variable-font support; update Pillow and rerun."; return 1; }
    nr_ok "Pillow is ready."
}

nr_prepare_fonts() {
    nr_section "JAPANESE FONT"
    nr_item "1" "Download Noto Sans JP and include Japanese bitmap fonts" \
        "Pinned GitHub revision, SHA-256 verified; about 10 MB download."
    nr_item "2" "Continue without Japanese fonts" \
        "The existing Spleen fonts remain in use."
    nr_ask "Choose Japanese font handling:"
    case "$REPLY" in
        2)
            export NIGHTRUN_DISABLE_JAPANESE_FONT=1
            nr_note "Japanese fonts skipped."
            return 0
            ;;
        1) unset NIGHTRUN_DISABLE_JAPANESE_FONT ;;
        *)
            nr_error "Choose 1 or 2."
            return 1
            ;;
    esac

    local log="$NR_LOG_DIR/fonts.log"
    mkdir -p -- "$(dirname -- "$log")"

    if ! command -v curl >/dev/null; then
        nr_error "curl is required to download the pinned font from GitHub."
        return 1
    fi
    if ! python3 -c 'from PIL import ImageFont; assert hasattr(ImageFont.FreeTypeFont, "get_variation_axes")' 2>/dev/null; then
        nr_offer_pillow || return 1
    fi

    nr_note "Downloading, verifying and rasterizing Noto Sans JP..."
    if ! bash "$NR_ROOT/scripts/installer/prepare-fonts.sh" >"$log" 2>&1; then
        nr_error "Noto Sans JP preparation failed. Last lines:"
        tail -8 "$log" | sed 's/^/    /'
        nr_note "Full log: $log"
        return 1
    fi
    nr_ok "Japanese PSF2 fonts prepared for 16, 24 and 32 pixels."
}