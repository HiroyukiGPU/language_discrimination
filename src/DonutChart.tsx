import { colorFor, LanguageStat } from "./types";

interface Props {
  languages: LanguageStat[];
  size?: number;
}

// SVG で言語割合のドーナツチャートを描く（外部ライブラリ不使用）。
export function DonutChart({ languages, size = 200 }: Props) {
  const stroke = size * 0.16;
  const radius = (size - stroke) / 2;
  const cx = size / 2;
  const cy = size / 2;
  const circumference = 2 * Math.PI * radius;

  const total = languages.reduce((s, l) => s + l.count, 0);
  let offset = 0;

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="donut">
      <g transform={`rotate(-90 ${cx} ${cy})`}>
        {total === 0 ? (
          <circle cx={cx} cy={cy} r={radius} fill="none" stroke="#21262d" strokeWidth={stroke} />
        ) : (
          languages.map((lang) => {
            const fraction = lang.count / total;
            const dash = fraction * circumference;
            const seg = (
              <circle
                key={lang.name}
                cx={cx}
                cy={cy}
                r={radius}
                fill="none"
                stroke={colorFor(lang.name)}
                strokeWidth={stroke}
                strokeDasharray={`${dash} ${circumference - dash}`}
                strokeDashoffset={-offset}
              >
                <title>{`${lang.name}: ${lang.count}件 (${lang.percentage.toFixed(1)}%)`}</title>
              </circle>
            );
            offset += dash;
            return seg;
          })
        )}
      </g>
      <text x={cx} y={cy - 6} textAnchor="middle" className="donut-total">
        {total}
      </text>
      <text x={cx} y={cy + 14} textAnchor="middle" className="donut-label">
        ファイル
      </text>
    </svg>
  );
}
