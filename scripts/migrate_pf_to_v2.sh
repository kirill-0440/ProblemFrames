#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <input.pf> [output.pf]" >&2
  exit 1
fi

INPUT="$1"
OUTPUT="${2:-}"

if [[ ! -f "${INPUT}" ]]; then
  echo "Input file not found: ${INPUT}" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "${TMP_OUT}"' EXIT

perl - "${INPUT}" > "${TMP_OUT}" <<'PERL'
use strict;
use warnings;

my ($input_path) = @ARGV;
open my $fh, "<", $input_path or die "failed to open $input_path: $!";
my @lines = <$fh>;
close $fh;

sub map_kind {
    my ($domain_type) = @_;
    return "causal" if $domain_type eq "Machine";
    return "causal" if $domain_type eq "Causal";
    return "biddable" if $domain_type eq "Biddable";
    return "lexical" if $domain_type eq "Lexical";
    return "causal" if $domain_type eq "Designed";
    return "causal";
}

sub map_role {
    my ($domain_type) = @_;
    return "machine" if $domain_type eq "Machine";
    return "designed" if $domain_type eq "Designed";
    return "given";
}

my $i = 0;
while ($i <= $#lines) {
    my $line = $lines[$i];

    if ($line =~ /^(\s*)domain\s+([A-Za-z][A-Za-z0-9_]*)\s+\[([A-Za-z]+)\](\s*\/\/.*)?\s*$/) {
        my ($indent, $name, $domain_type, $comment) = ($1, $2, $3, $4 // "");
        my $kind = map_kind($domain_type);
        my $role = map_role($domain_type);
        print "${indent}domain ${name} kind ${kind} role ${role}${comment}\n";
        $i++;
        next;
    }

    if ($line =~ /^(\s*)interface\s+"([^"]+)"\s*\{\s*$/) {
        my ($indent, $name) = ($1, $2);
        my $depth = 1;
        my @block_lines;
        $i++;

        while ($i <= $#lines && $depth > 0) {
            my $block_line = $lines[$i];
            my $open_count = () = $block_line =~ /\{/g;
            my $close_count = () = $block_line =~ /\}/g;
            $depth += $open_count - $close_count;
            push @block_lines, $block_line;
            $i++;
        }

        my @phenomena;
        my @connects;
        my %seen_connect;
        for my $block_line (@block_lines) {
            if ($block_line =~ /^\s*(event|command|state|value)\s+([A-Za-z][A-Za-z0-9_]*)\s*\[\s*([A-Za-z][A-Za-z0-9_]*)\s*->\s*([A-Za-z][A-Za-z0-9_]*)\s*\]\s*$/) {
                my ($type, $pname, $from, $to) = ($1, $2, $3, $4);
                push @phenomena, "phenomenon ${pname} : ${type} [${from} -> ${to}] controlledBy ${from}";
                if (!$seen_connect{$from}) {
                    $seen_connect{$from} = 1;
                    push @connects, $from;
                }
                if (!$seen_connect{$to}) {
                    $seen_connect{$to} = 1;
                    push @connects, $to;
                }
            }
        }

        if (!@connects) {
            @connects = ("TODO_A", "TODO_B");
        }

        print "${indent}interface \"${name}\" connects " . join(", ", @connects) . " {\n";
        print "${indent}    shared: {\n";
        for my $phenomenon (@phenomena) {
            print "${indent}        ${phenomenon}\n";
        }
        print "${indent}    }\n";
        print "${indent}}\n";
        next;
    }

    print $line;
    $i++;
}
PERL

if [[ -n "${OUTPUT}" ]]; then
  mv "${TMP_OUT}" "${OUTPUT}"
  trap - EXIT
else
  cat "${TMP_OUT}"
fi
