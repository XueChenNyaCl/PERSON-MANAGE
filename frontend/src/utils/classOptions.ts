export interface GradeOption {
  label: string
  value: number
}

export const gradeOptions: GradeOption[] = [
  { label: '一年级', value: 1 },
  { label: '二年级', value: 2 },
  { label: '三年级', value: 3 },
  { label: '四年级', value: 4 },
  { label: '五年级', value: 5 },
  { label: '六年级', value: 6 },
  { label: '七年级', value: 7 },
  { label: '八年级', value: 8 },
  { label: '九年级', value: 9 },
  { label: '高一', value: 10 },
  { label: '高二', value: 11 },
  { label: '高三', value: 12 }
]

export const getGradeLabel = (grade?: number | null) => {
  const matched = gradeOptions.find((option) => option.value === grade)
  return matched?.label ?? (grade ? `${grade}年级` : '-')
}