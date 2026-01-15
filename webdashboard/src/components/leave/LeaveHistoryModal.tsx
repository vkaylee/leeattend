import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { format } from "date-fns";
import { Loader2, Plus, Minus, History } from "lucide-react";
import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import { adjustLeaveBalance, fetchLeaveHistory, LeaveHistoryItem } from "@/lib/api";
import { toast } from "sonner";

interface LeaveHistoryModalProps {
  wsId: string;
  employeeId: string;
  employeeName: string;
  isOpen: boolean;
  onClose: () => void;
}

export function LeaveHistoryModal({ wsId, employeeId, employeeName, isOpen, onClose }: LeaveHistoryModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState("history");

  // Fetch History
  const { data: history, isLoading } = useQuery({
    queryKey: ["leave-history", wsId, employeeId],
    queryFn: () => fetchLeaveHistory(wsId, employeeId),
    enabled: isOpen,
  });

  // Adjust Mutation
  const adjustMutation = useMutation({
    mutationFn: (data: { leave_type: string; amount: number; reason: string; transaction_date: string }) =>
      adjustLeaveBalance(wsId, employeeId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["leave-history", wsId, employeeId] });
      queryClient.invalidateQueries({ queryKey: ["leave-balances", wsId, employeeId] });
      toast.success("Balance adjusted successfully");
      setActiveTab("history");
    },
    onError: (error) => {
      toast.error("Failed to adjust balance: " + error.message);
    }
  });

  const [adjustForm, setAdjustForm] = useState({
    type: "add", // add or deduct
    leave_type: "vacation",
    amount: "",
    reason: "",
    date: new Date().toISOString().split('T')[0]
  });

  const handleAdjustSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(adjustForm.amount);
    if (isNaN(amount) || amount <= 0) {
      toast.error("Invalid amount");
      return;
    }

    const finalAmount = adjustForm.type === "add" ? amount : -amount;
    
    adjustMutation.mutate({
      leave_type: adjustForm.leave_type,
      amount: finalAmount,
      reason: adjustForm.reason,
      transaction_date: adjustForm.date
    });
  };

  const getTransactionColor = (type: string) => {
    switch (type) {
      case "accrual": return "bg-green-100 text-green-700";
      case "usage": return "bg-red-100 text-red-700";
      case "adjustment": return "bg-amber-100 text-amber-700";
      case "carry_over": return "bg-blue-100 text-blue-700";
      default: return "bg-slate-100 text-slate-700";
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[700px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Leave History: {employeeName}</DialogTitle>
          <DialogDescription>View detailed history and adjust leave balances.</DialogDescription>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="mt-4">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="history">History</TabsTrigger>
            <TabsTrigger value="adjust">Manual Adjustment</TabsTrigger>
          </TabsList>

          <TabsContent value="history" className="space-y-4">
            {isLoading ? (
              <div className="flex justify-center p-8"><Loader2 className="animate-spin" /></div>
            ) : history && history.length > 0 ? (
              <div className="border rounded-md">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Date</TableHead>
                      <TableHead>Type</TableHead>
                      <TableHead className="text-right">Amount</TableHead>
                      <TableHead>Reason</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {history.map((item) => (
                      <TableRow key={item.id}>
                        <TableCell>{format(new Date(item.date), "MMM dd, yyyy")}</TableCell>
                        <TableCell>
                          <Badge variant="outline" className={getTransactionColor(item.transaction_type)}>
                            {item.transaction_type.toUpperCase()}
                          </Badge>
                        </TableCell>
                        <TableCell className={`text-right font-bold ${item.amount > 0 ? "text-green-600" : "text-red-600"}`}>
                          {item.amount > 0 ? "+" : ""}{item.amount}
                        </TableCell>
                        <TableCell className="max-w-[200px] truncate" title={item.reason || ""}>
                          {item.reason || "-"}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            ) : (
              <div className="text-center p-8 text-slate-500 italic">No history records found.</div>
            )}
          </TabsContent>

          <TabsContent value="adjust" className="space-y-4 py-4">
            <form onSubmit={handleAdjustSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Action</Label>
                  <Select value={adjustForm.type} onValueChange={(v) => setAdjustForm({...adjustForm, type: v})}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="add"><div className="flex items-center"><Plus className="w-4 h-4 mr-2 text-green-600" /> Add Balance</div></SelectItem>
                      <SelectItem value="deduct"><div className="flex items-center"><Minus className="w-4 h-4 mr-2 text-red-600" /> Deduct Balance</div></SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>Leave Type</Label>
                  <Select value={adjustForm.leave_type} onValueChange={(v) => setAdjustForm({...adjustForm, leave_type: v})}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="vacation">Annual Leave</SelectItem>
                      <SelectItem value="sick">Sick Leave</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Amount (Days)</Label>
                  <Input 
                    type="number" 
                    step="0.5" 
                    min="0.5"
                    required
                    value={adjustForm.amount} 
                    onChange={(e) => setAdjustForm({...adjustForm, amount: e.target.value})} 
                    placeholder="e.g. 1.0"
                  />
                </div>
                <div className="space-y-2">
                  <Label>Date</Label>
                  <Input 
                    type="date" 
                    required
                    value={adjustForm.date} 
                    onChange={(e) => setAdjustForm({...adjustForm, date: e.target.value})} 
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label>Reason (Required for audit)</Label>
                <Textarea 
                  required
                  placeholder="Explain why this adjustment is made..."
                  value={adjustForm.reason}
                  onChange={(e) => setAdjustForm({...adjustForm, reason: e.target.value})}
                />
              </div>

              <div className="flex justify-end pt-4">
                <Button type="submit" disabled={adjustMutation.isPending} className="bg-slate-900 text-white">
                  {adjustMutation.isPending ? <Loader2 className="animate-spin w-4 h-4 mr-2" /> : <History className="w-4 h-4 mr-2" />}
                  Confirm Adjustment
                </Button>
              </div>
            </form>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
